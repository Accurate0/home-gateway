use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use home_gateway::auth::api_types::{ApiKeyInfo, CreateKeyPayload, CreatedKey, UpdateKeyPayload};
use home_gateway::cli::client::{Client, DEFAULT_BASE_URL, REQUEST_TIMEOUT};
use home_gateway::cli::credentials;
use home_gateway::cli::oauth::{self, DEFAULT_CLIENT_ID, DEFAULT_ISSUER};
use home_gateway::http::get_traced_http_client;
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "home", about = "control the home gateway", version)]
struct Cli {
    #[arg(long, env = "HG_BASE_URL", default_value = DEFAULT_BASE_URL, global = true)]
    base_url: String,
    #[arg(long, env = "HG_API_KEY", global = true)]
    api_key: Option<String>,
    #[arg(long, env = "HG_ISSUER", default_value = DEFAULT_ISSUER, global = true)]
    issuer: String,
    #[arg(long, env = "HG_CLIENT_ID", default_value = DEFAULT_CLIENT_ID, global = true)]
    client_id: String,
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Login,
    Logout,
    Whoami,
    Ls(LsArgs),
    #[command(subcommand)]
    Light(LightCommand),
    #[command(subcommand)]
    Workflow(WorkflowCommand),
    #[command(subcommand)]
    Mode(ModeCommand),
    #[command(subcommand)]
    Keys(KeysCommand),
    Push(PushArgs),
    Curl(CurlArgs),
}

#[derive(Args)]
struct LsArgs {
    #[arg(long, value_enum)]
    category: Option<Category>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Category {
    Lights,
    Doors,
    Presence,
    Environment,
    Displays,
    Vacuums,
    Media,
}

impl Category {
    fn as_graphql(self) -> &'static str {
        match self {
            Self::Lights => "LIGHTS",
            Self::Doors => "DOORS",
            Self::Presence => "PRESENCE",
            Self::Environment => "ENVIRONMENT",
            Self::Displays => "DISPLAYS",
            Self::Vacuums => "VACUUMS",
            Self::Media => "MEDIA",
        }
    }
}

#[derive(Subcommand)]
enum LightCommand {
    On { id: String },
    Off { id: String },
    Toggle { id: String },
    Set(LightSetArgs),
}

#[derive(Args)]
struct LightSetArgs {
    id: String,
    #[arg(long)]
    on: bool,
    #[arg(long, conflicts_with = "on")]
    off: bool,
    #[arg(long)]
    brightness: Option<i64>,
    #[arg(long)]
    colour_temperature: Option<i64>,
    #[arg(long)]
    colour: Option<String>,
}

#[derive(Subcommand)]
enum WorkflowCommand {
    List,
    Run {
        slug: String,
        #[arg(long = "input", value_parser = parse_input)]
        inputs: Vec<(String, Value)>,
    },
    Exec {
        file: PathBuf,
        #[arg(long = "input", value_parser = parse_input)]
        inputs: Vec<(String, Value)>,
    },
    Enable {
        slug: String,
    },
    Disable {
        slug: String,
    },
    Runs {
        #[arg(long)]
        slug: Option<String>,
        #[arg(long)]
        limit: Option<i64>,
    },
}

fn parse_input(raw: &str) -> Result<(String, Value), String> {
    let (key, value) = raw
        .split_once('=')
        .ok_or_else(|| format!("expected key=value, got `{raw}`"))?;

    let value = serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_owned()));

    Ok((key.to_owned(), value))
}

fn input_map(inputs: &[(String, Value)]) -> Value {
    Value::Object(inputs.iter().cloned().collect())
}

#[derive(Subcommand)]
enum ModeCommand {
    Show,
    Set { mode: String },
}

#[derive(Subcommand)]
enum KeysCommand {
    Create {
        name: String,
        #[arg(value_delimiter = ',')]
        scopes: Vec<String>,
        #[arg(long)]
        expires_at: Option<DateTime<Utc>>,
    },
    List,
    Update {
        id: Uuid,
        #[arg(long)]
        name: Option<String>,
        #[arg(long, value_delimiter = ',')]
        scopes: Option<Vec<String>>,
        #[arg(long)]
        expires_at: Option<DateTime<Utc>>,
    },
    Regenerate {
        id: Uuid,
    },
    Revoke {
        id: Uuid,
    },
}

#[derive(Args)]
struct PushArgs {
    body: String,
    #[arg(long, default_value = "Home Gateway")]
    title: String,
    #[arg(long, value_enum, default_value_t = PushCategory::General)]
    category: PushCategory,
    #[arg(long)]
    tag: Option<String>,
    #[arg(
        long = "action",
        value_name = "LABEL=KIND",
        value_parser = parse_push_action,
        help = "repeatable; KIND is acknowledge, dismiss, snooze:SECONDS or workflow:SLUG"
    )]
    actions: Vec<Value>,
    #[arg(long, value_name = "DURATION", value_parser = parse_remind_after, requires = "reminders")]
    remind_after: Option<i64>,
    #[arg(long, requires = "remind_after")]
    reminders: Option<i32>,
}

#[derive(Clone, Copy, ValueEnum)]
enum PushCategory {
    Alarm,
    Door,
    Watchdog,
    General,
}

impl PushCategory {
    fn as_graphql(self) -> &'static str {
        match self {
            Self::Alarm => "ALARM",
            Self::Door => "DOOR",
            Self::Watchdog => "WATCHDOG",
            Self::General => "GENERAL",
        }
    }
}

#[derive(Args)]
struct CurlArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
    args: Vec<String>,
}

const SEND_PUSH_MUTATION: &str =
    "mutation($input: SendPushNotificationInput!) { sendPushNotification(input: $input) }";

const CURL_AUTH_HEADER_VARIABLE: &str = "HG_CURL_AUTH_HEADER";

const WHOAMI_QUERY: &str = "query { auth { id name scopes } }";

const ENTITIES_QUERY: &str = r#"
query {
  entities {
    __typename
    ... on LightEntity { id name category room on }
    ... on DoorEntity { id name category room open }
    ... on PresenceEntity { id name category room }
    ... on EnvironmentEntity { id name category room }
    ... on EinkDisplayEntity { id name category room }
    ... on RobotVacuumEntity { id name category room }
    ... on MediaPlayerEntity { id name category room }
  }
}
"#;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Command::Login => login(&cli).await,
        Command::Logout => logout(),
        _ => run(&cli).await,
    }
}

async fn login(cli: &Cli) -> Result<()> {
    let http = get_traced_http_client(REQUEST_TIMEOUT)?;
    let credentials = oauth::login(&http, &cli.issuer, &cli.client_id).await?;
    let path = credentials::store(&credentials)?;

    println!("credentials written to {}", path.display());

    let client = Client::new(&cli.base_url, None).await?;
    let data = client.graphql(WHOAMI_QUERY, json!({})).await?;
    print_whoami(&data["auth"], cli.json)
}

fn logout() -> Result<()> {
    if credentials::clear()? {
        println!("logged out");
    } else {
        println!("not logged in");
    }

    Ok(())
}

async fn run(cli: &Cli) -> Result<()> {
    let client = Client::new(&cli.base_url, cli.api_key.clone()).await?;

    match &cli.command {
        Command::Login | Command::Logout => unreachable!("handled before building a client"),
        Command::Whoami => {
            let data = client.graphql(WHOAMI_QUERY, json!({})).await?;
            print_whoami(&data["auth"], cli.json)
        }
        Command::Ls(args) => list_entities(&client, args, cli.json).await,
        Command::Light(command) => light(&client, command, cli.json).await,
        Command::Workflow(command) => workflow(&client, command, cli.json).await,
        Command::Mode(command) => mode(&client, command, cli.json).await,
        Command::Keys(command) => keys(&client, command, cli.json).await,
        Command::Push(args) => push(&client, args, cli.json).await,
        Command::Curl(args) => curl(&client, args),
    }
}

fn print_whoami(auth: &Value, as_json: bool) -> Result<()> {
    if as_json {
        return print_json(auth);
    }

    let name = auth["name"].as_str().unwrap_or("unknown");
    println!("signed in as {name}");

    let scopes = auth["scopes"].as_array().cloned().unwrap_or_default();
    if scopes.is_empty() {
        println!("no scopes granted");
    } else {
        println!("scopes:");
        for scope in scopes {
            println!("  {}", scope.as_str().unwrap_or_default());
        }
    }

    Ok(())
}

async fn list_entities(client: &Client, args: &LsArgs, as_json: bool) -> Result<()> {
    let data = client.graphql(ENTITIES_QUERY, json!({})).await?;
    let entities = data["entities"].as_array().cloned().unwrap_or_default();

    let entities: Vec<Value> = match args.category {
        Some(category) => entities
            .into_iter()
            .filter(|e| e["category"] == category.as_graphql())
            .collect(),
        None => entities,
    };

    if as_json {
        return print_json(&Value::Array(entities));
    }

    if entities.is_empty() {
        println!("no entities, note that kinds your scopes do not cover are omitted silently");
        return Ok(());
    }

    for entity in &entities {
        let category = entity["category"].as_str().unwrap_or("");
        let id = entity["id"].as_str().unwrap_or("");
        let name = entity["name"].as_str().unwrap_or("");
        let room = entity["room"].as_str().unwrap_or("-");

        println!(
            "{category:<12} {id:<28} {name:<28} {room:<16} {}",
            state(entity)
        );
    }

    Ok(())
}

fn state(entity: &Value) -> String {
    if let Some(on) = entity["on"].as_bool() {
        return if on { "on" } else { "off" }.to_owned();
    }

    match entity["open"].as_bool() {
        Some(true) => "open".to_owned(),
        Some(false) => "closed".to_owned(),
        None => String::new(),
    }
}

fn set_light_query(args: &LightSetArgs) -> Result<(String, Value)> {
    let mut declarations = vec!["$id: String!".to_owned()];
    let mut selections = Vec::new();
    let mut variables = serde_json::Map::new();

    variables.insert("id".to_owned(), json!(args.id));

    if args.on {
        selections.push("power: on".to_owned());
    }
    if args.off {
        selections.push("power: off".to_owned());
    }

    if let Some(brightness) = args.brightness {
        declarations.push("$brightness: SetBrightnessInput!".to_owned());
        selections.push("brightness: setBrightness(input: $brightness)".to_owned());
        variables.insert("brightness".to_owned(), json!({ "value": brightness }));
    }

    if let Some(colour_temperature) = args.colour_temperature {
        declarations.push("$colourTemperature: SetColourTemperatureInput!".to_owned());
        selections
            .push("colourTemperature: setColourTemperature(input: $colourTemperature)".to_owned());
        variables.insert(
            "colourTemperature".to_owned(),
            json!({ "value": colour_temperature }),
        );
    }

    if let Some(colour) = &args.colour {
        declarations.push("$colour: SetColourInput!".to_owned());
        selections.push("colour: setColour(input: $colour)".to_owned());
        variables.insert("colour".to_owned(), json!({ "hex": colour }));
    }

    if selections.is_empty() {
        anyhow::bail!(
            "nothing to set, pass at least one of --on/--off, --brightness, --colour-temperature or --colour"
        );
    }

    let query = format!(
        "mutation({}) {{ light(id: $id) {{ {} }} }}",
        declarations.join(", "),
        selections.join(" ")
    );

    Ok((query, Value::Object(variables)))
}

async fn light(client: &Client, command: &LightCommand, as_json: bool) -> Result<()> {
    let data = match command {
        LightCommand::On { id } => {
            client
                .graphql(
                    "mutation($id: String!) { light(id: $id) { on } }",
                    json!({ "id": id }),
                )
                .await?
        }
        LightCommand::Off { id } => {
            client
                .graphql(
                    "mutation($id: String!) { light(id: $id) { off } }",
                    json!({ "id": id }),
                )
                .await?
        }
        LightCommand::Toggle { id } => {
            client
                .graphql(
                    "mutation($id: String!) { light(id: $id) { toggle } }",
                    json!({ "id": id }),
                )
                .await?
        }
        LightCommand::Set(args) => {
            let (query, variables) = set_light_query(args)?;
            client.graphql(&query, variables).await?
        }
    };

    if as_json {
        return print_json(&data["light"]);
    }

    match command {
        LightCommand::Set(_) => {
            let applied = data["light"]
                .as_object()
                .map(|fields| fields.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default();

            println!("applied {}", applied.join(", "));
        }
        _ => println!("ok"),
    }

    Ok(())
}

async fn workflow(client: &Client, command: &WorkflowCommand, as_json: bool) -> Result<()> {
    match command {
        WorkflowCommand::List => {
            let data = client
                .graphql(
                    "query { workflows { slug name group tags enabled configEnabled dryRun reusable } }",
                    json!({}),
                )
                .await?;

            if as_json {
                return print_json(&data["workflows"]);
            }

            let workflows = data["workflows"].as_array().cloned().unwrap_or_default();
            for workflow in &workflows {
                let enabled = if workflow["enabled"].as_bool().unwrap_or(false) {
                    "enabled"
                } else {
                    "disabled"
                };
                let dry_run = if workflow["dryRun"].as_bool().unwrap_or(false) {
                    " (dry-run)"
                } else {
                    ""
                };

                println!(
                    "{:<10} {:<32} {:<20}{dry_run}",
                    enabled,
                    workflow["slug"].as_str().unwrap_or(""),
                    workflow["group"].as_str().unwrap_or("")
                );
            }

            Ok(())
        }
        WorkflowCommand::Run { slug, inputs } => {
            let data = client
                .graphql(
                    "mutation($slug: String!, $inputs: JSON) { runWorkflow(slug: $slug, inputs: $inputs) }",
                    json!({ "slug": slug, "inputs": input_map(inputs) }),
                )
                .await?;

            report(&data, as_json, &format!("ran {slug}"))
        }
        WorkflowCommand::Exec { file, inputs } => {
            let source = std::fs::read_to_string(file)
                .with_context(|| format!("failed to read {}", file.display()))?;
            let workflow: Value = serde_yaml::from_str(&source)
                .with_context(|| format!("failed to parse {}", file.display()))?;

            let response = client
                .send(
                    Method::POST,
                    "/v1/workflow/execute",
                    Some(&json!({ "workflow": workflow, "inputs": input_map(inputs) })),
                )
                .await?;

            let status = response.status();
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                anyhow::bail!("workflow execution failed with {status}: {body}");
            }

            report(&json!({ "dispatched": true }), as_json, "dispatched")
        }
        WorkflowCommand::Enable { slug } => set_enabled(client, slug, true, as_json).await,
        WorkflowCommand::Disable { slug } => set_enabled(client, slug, false, as_json).await,
        WorkflowCommand::Runs { slug, limit } => {
            let data = client
                .graphql(
                    "query($slug: String, $limit: Int) { workflowRuns(slug: $slug, limit: $limit) { id slug name outcome dryRun durationMs error startedAt } }",
                    json!({ "slug": slug, "limit": limit }),
                )
                .await?;

            print_json(&data["workflowRuns"])
        }
    }
}

async fn set_enabled(client: &Client, slug: &str, enabled: bool, as_json: bool) -> Result<()> {
    let data = client
        .graphql(
            "mutation($slug: String!, $enabled: Boolean!) { setWorkflowEnabled(slug: $slug, enabled: $enabled) }",
            json!({ "slug": slug, "enabled": enabled }),
        )
        .await?;

    let message = if enabled {
        format!("enabled {slug}")
    } else {
        format!("disabled {slug}")
    };

    report(&data, as_json, &message)
}

async fn mode(client: &Client, command: &ModeCommand, as_json: bool) -> Result<()> {
    match command {
        ModeCommand::Show => {
            let data = client
                .graphql("query { mode { active } }", json!({}))
                .await?;

            if as_json {
                return print_json(&data["mode"]["active"]);
            }

            println!(
                "{}",
                data["mode"]["active"]
                    .as_str()
                    .unwrap_or_default()
                    .to_lowercase()
            );

            Ok(())
        }
        ModeCommand::Set { mode } => {
            let data = client
                .graphql(
                    "mutation($mode: Mode!) { setMode(mode: $mode) }",
                    json!({ "mode": mode.to_uppercase() }),
                )
                .await?;

            if as_json {
                return print_json(&data["setMode"]);
            }

            println!(
                "mode is now {}",
                data["setMode"].as_str().unwrap_or_default().to_lowercase()
            );

            Ok(())
        }
    }
}

async fn keys(client: &Client, command: &KeysCommand, as_json: bool) -> Result<()> {
    match command {
        KeysCommand::Create {
            name,
            scopes,
            expires_at,
        } => {
            let payload = CreateKeyPayload {
                name: name.clone(),
                scopes: scopes.clone(),
                expires_at: *expires_at,
            };

            let created: CreatedKey = client
                .json(Method::POST, "/v1/admin/keys", Some(&payload))
                .await?;

            print_created_key(&created, as_json)
        }
        KeysCommand::List => {
            let keys: Vec<ApiKeyInfo> = client.json(Method::GET, "/v1/admin/keys", NO_BODY).await?;

            if as_json {
                println!("{}", serde_json::to_string_pretty(&keys)?);
                return Ok(());
            }

            for key in &keys {
                let status = if key.revoked_at.is_some() {
                    "revoked"
                } else {
                    "active"
                };

                println!(
                    "{:<8} {} {:<28} {}",
                    status,
                    key.id,
                    key.name,
                    key.scopes.join(",")
                );
            }

            Ok(())
        }
        KeysCommand::Update {
            id,
            name,
            scopes,
            expires_at,
        } => {
            let payload = UpdateKeyPayload {
                name: name.clone(),
                scopes: scopes.clone(),
                expires_at: *expires_at,
            };

            let response = client
                .send(
                    Method::PATCH,
                    &format!("/v1/admin/keys/{id}"),
                    Some(&payload),
                )
                .await?;

            if response.status() == StatusCode::NOT_FOUND {
                println!("no active key with id {id}");
                return Ok(());
            }

            let info: ApiKeyInfo = response.json().await.context("failed to read the key")?;
            println!("{}", serde_json::to_string_pretty(&info)?);
            Ok(())
        }
        KeysCommand::Regenerate { id } => {
            let response = client
                .send(
                    Method::POST,
                    &format!("/v1/admin/keys/{id}/regenerate"),
                    NO_BODY,
                )
                .await?;

            if response.status() == StatusCode::NOT_FOUND {
                println!("no active key with id {id}");
                return Ok(());
            }

            let created: CreatedKey = response.json().await.context("failed to read the key")?;
            print_created_key(&created, as_json)
        }
        KeysCommand::Revoke { id } => {
            let response = client
                .send(Method::DELETE, &format!("/v1/admin/keys/{id}"), NO_BODY)
                .await?;

            if response.status() == StatusCode::NOT_FOUND {
                println!("no active key with id {id}");
            } else {
                println!("revoked {id}");
            }

            Ok(())
        }
    }
}

const NO_BODY: Option<&Value> = None;

fn print_created_key(created: &CreatedKey, as_json: bool) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(created)?);

    if !as_json {
        println!("\nstore this key now, it will not be shown again");
    }

    Ok(())
}

fn parse_push_action(spec: &str) -> Result<Value, String> {
    let (label, action) = spec
        .split_once('=')
        .ok_or_else(|| format!("expected LABEL=KIND, got '{spec}'"))?;

    if label.is_empty() {
        return Err("action label is empty".to_owned());
    }

    let (kind, argument) = match action.split_once(':') {
        Some((kind, argument)) => (kind, Some(argument)),
        None => (action, None),
    };

    match (kind, argument) {
        ("acknowledge", None) => Ok(json!({ "label": label, "kind": "ACKNOWLEDGE" })),
        ("dismiss", None) => Ok(json!({ "label": label, "kind": "DISMISS" })),
        ("snooze", Some(seconds)) => {
            let seconds: i64 = seconds
                .parse()
                .map_err(|_| format!("snooze needs whole seconds, got '{seconds}'"))?;

            Ok(json!({ "label": label, "kind": "SNOOZE", "snoozeSeconds": seconds }))
        }
        ("workflow", Some(slug)) if !slug.is_empty() => {
            Ok(json!({ "label": label, "kind": "RUN_WORKFLOW", "workflowSlug": slug }))
        }
        _ => Err(format!(
            "unknown action '{action}', expected acknowledge, dismiss, snooze:SECONDS or workflow:SLUG"
        )),
    }
}

fn parse_remind_after(value: &str) -> Result<i64, String> {
    home_gateway::timedelta_format::parse_datetime_str_with_ms(value)
        .map(|delta| delta.num_seconds())
        .map_err(|e| e.to_string())
}

fn push_input(args: &PushArgs) -> Value {
    let mut input = json!({
        "title": args.title,
        "body": args.body,
        "category": args.category.as_graphql(),
        "tag": args.tag,
        "actions": args.actions,
    });

    if let (Some(remind_after), Some(reminders)) = (args.remind_after, args.reminders) {
        input["acknowledge"] = json!({
            "remindAfterSeconds": remind_after,
            "reminders": reminders,
        });
    }

    input
}

async fn push(client: &Client, args: &PushArgs, as_json: bool) -> Result<()> {
    let data = client
        .graphql(SEND_PUSH_MUTATION, json!({ "input": push_input(args) }))
        .await?;

    report(&data, as_json, "sent")
}

fn curl_args(base_url: &str, args: &[String]) -> Vec<String> {
    let mut out = vec![
        "--variable".to_owned(),
        format!("%{CURL_AUTH_HEADER_VARIABLE}"),
        "--expand-header".to_owned(),
        format!("{{{{{CURL_AUTH_HEADER_VARIABLE}}}}}"),
    ];

    out.extend(args.iter().map(|arg| {
        if arg.starts_with("/v1/") {
            format!("{base_url}{arg}")
        } else {
            arg.clone()
        }
    }));

    out
}

fn curl(client: &Client, args: &CurlArgs) -> Result<()> {
    let status = std::process::Command::new("curl")
        .args(curl_args(client.base_url(), &args.args))
        .env(CURL_AUTH_HEADER_VARIABLE, client.auth_header())
        .status()
        .context("failed to run curl, is it installed?")?;

    std::process::exit(status.code().unwrap_or(1));
}

fn report(data: &Value, as_json: bool, message: &str) -> Result<()> {
    if as_json {
        return print_json(data);
    }

    println!("{message}");
    Ok(())
}

fn print_json(value: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(id: &str) -> LightSetArgs {
        LightSetArgs {
            id: id.to_owned(),
            on: false,
            off: false,
            brightness: None,
            colour_temperature: None,
            colour: None,
        }
    }

    #[test]
    fn setting_nothing_is_rejected() {
        assert!(set_light_query(&args("lamp")).is_err());
    }

    #[test]
    fn brightness_is_sent_as_a_typed_variable() {
        let mut args = args("lamp");
        args.brightness = Some(120);

        let (query, variables) = set_light_query(&args).unwrap();

        assert!(query.contains("$brightness: SetBrightnessInput!"));
        assert!(query.contains("brightness: setBrightness(input: $brightness)"));
        assert_eq!(variables["brightness"]["value"], 120);
        assert_eq!(variables["id"], "lamp");
    }

    #[test]
    fn colour_uses_the_hex_input_field() {
        let mut args = args("lamp");
        args.colour = Some("#ff0000".to_owned());

        let (query, variables) = set_light_query(&args).unwrap();

        assert!(query.contains("colour: setColour(input: $colour)"));
        assert_eq!(variables["colour"]["hex"], "#ff0000");
    }

    #[test]
    fn power_and_every_attribute_ride_in_one_mutation() {
        let mut args = args("lamp");
        args.on = true;
        args.brightness = Some(10);
        args.colour_temperature = Some(300);
        args.colour = Some("#abcdef".to_owned());

        let (query, variables) = set_light_query(&args).unwrap();

        assert!(query.starts_with("mutation($id: String!, "));
        assert!(query.contains("power: on"));
        assert!(
            query.contains("colourTemperature: setColourTemperature(input: $colourTemperature)")
        );
        assert_eq!(variables["colourTemperature"]["value"], 300);
        assert_eq!(query.matches("light(id: $id)").count(), 1);
    }

    #[test]
    fn turning_off_selects_the_off_field() {
        let mut args = args("lamp");
        args.off = true;

        let (query, _) = set_light_query(&args).unwrap();
        assert!(query.contains("power: off"));
    }

    #[test]
    fn a_light_entity_renders_its_power_state() {
        assert_eq!(state(&json!({ "on": true })), "on");
        assert_eq!(state(&json!({ "on": false })), "off");
    }

    #[test]
    fn a_door_entity_renders_its_open_state() {
        assert_eq!(state(&json!({ "open": true })), "open");
        assert_eq!(state(&json!({ "open": false })), "closed");
    }

    #[test]
    fn an_entity_with_no_readable_state_renders_empty() {
        assert_eq!(state(&json!({ "id": "x" })), "");
    }

    #[test]
    fn push_actions_parse_every_kind() {
        assert_eq!(
            parse_push_action("Done=acknowledge").unwrap(),
            json!({ "label": "Done", "kind": "ACKNOWLEDGE" })
        );
        assert_eq!(
            parse_push_action("Later=snooze:300").unwrap(),
            json!({ "label": "Later", "kind": "SNOOZE", "snoozeSeconds": 300 })
        );
        assert_eq!(
            parse_push_action("Lamps=workflow:living-room-lamps-on").unwrap(),
            json!({ "label": "Lamps", "kind": "RUN_WORKFLOW", "workflowSlug": "living-room-lamps-on" })
        );
        assert_eq!(
            parse_push_action("Close=dismiss").unwrap(),
            json!({ "label": "Close", "kind": "DISMISS" })
        );
    }

    #[test]
    fn malformed_push_actions_are_rejected() {
        for spec in [
            "acknowledge",
            "=dismiss",
            "Later=snooze",
            "Later=snooze:soon",
            "Go=workflow:",
            "X=explode",
        ] {
            assert!(
                parse_push_action(spec).is_err(),
                "{spec} should be rejected"
            );
        }
    }

    #[test]
    fn push_input_carries_the_acknowledge_policy() {
        let args = PushArgs {
            body: "Bins".to_owned(),
            title: "Home Gateway".to_owned(),
            category: PushCategory::General,
            tag: None,
            actions: vec![parse_push_action("Done=acknowledge").unwrap()],
            remind_after: Some(parse_remind_after("2h").unwrap()),
            reminders: Some(1),
        };

        let input = push_input(&args);

        assert_eq!(input["category"], "GENERAL");
        assert_eq!(input["actions"][0]["kind"], "ACKNOWLEDGE");
        assert_eq!(input["acknowledge"]["remindAfterSeconds"], 7200);
        assert_eq!(input["acknowledge"]["reminders"], 1);
    }

    #[test]
    fn curl_prefixes_gateway_paths_and_injects_the_auth_header() {
        let args: Vec<String> = ["-s", "-o", "/tmp/out.json", "/v1/health"]
            .into_iter()
            .map(str::to_owned)
            .collect();

        let out = curl_args("https://home.anurag.sh", &args);

        assert_eq!(
            out,
            vec![
                "--variable",
                "%HG_CURL_AUTH_HEADER",
                "--expand-header",
                "{{HG_CURL_AUTH_HEADER}}",
                "-s",
                "-o",
                "/tmp/out.json",
                "https://home.anurag.sh/v1/health",
            ]
        );
    }
}
