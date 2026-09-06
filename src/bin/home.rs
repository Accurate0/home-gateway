use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use home_gateway::auth::api_types::{ApiKeyInfo, CreateKeyPayload, CreatedKey, UpdateKeyPayload};
use home_gateway::cli::client::{Client, ClientError, DEFAULT_BASE_URL};
use home_gateway::cli::credentials;
use home_gateway::cli::oauth::{self, DEFAULT_CLIENT_ID, DEFAULT_ISSUER};
use home_gateway::http::get_traced_http_client;
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
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

#[derive(Subcommand)]
enum ModeCommand {
    List,
    Set {
        mode: String,
        #[arg(long, conflicts_with = "inactive")]
        active: bool,
        #[arg(long)]
        inactive: bool,
    },
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
    #[arg(long)]
    title: Option<String>,
    #[arg(long)]
    category: Option<String>,
    #[arg(long)]
    tag: Option<String>,
}

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
    let http = get_traced_http_client()?;
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
        Command::Push(args) => push(&client, args).await,
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
        WorkflowCommand::Run { slug } => {
            let data = client
                .graphql(
                    "mutation($slug: String!) { runWorkflow(slug: $slug) }",
                    json!({ "slug": slug }),
                )
                .await?;

            report(&data, as_json, &format!("ran {slug}"))
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
        ModeCommand::List => {
            let data = client.graphql("query { activeModes }", json!({})).await?;

            if as_json {
                return print_json(&data["activeModes"]);
            }

            let modes = data["activeModes"].as_array().cloned().unwrap_or_default();
            if modes.is_empty() {
                println!("no active modes");
            } else {
                for mode in modes {
                    println!("{}", mode.as_str().unwrap_or_default());
                }
            }

            Ok(())
        }
        ModeCommand::Set {
            mode,
            active,
            inactive,
        } => {
            if !active && !inactive {
                anyhow::bail!("pass either --active or --inactive");
            }

            let data = client
                .graphql(
                    "mutation($mode: Mode!, $active: Boolean!) { setMode(mode: $mode, active: $active) }",
                    json!({ "mode": mode.to_uppercase(), "active": *active }),
                )
                .await?;

            if as_json {
                return print_json(&data["setMode"]);
            }

            println!("active modes: {}", data["setMode"]);
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

async fn push(client: &Client, args: &PushArgs) -> Result<()> {
    let mut payload = serde_json::Map::new();
    payload.insert("body".to_owned(), json!(args.body));

    if let Some(title) = &args.title {
        payload.insert("title".to_owned(), json!(title));
    }
    if let Some(category) = &args.category {
        payload.insert("category".to_owned(), json!(category));
    }
    if let Some(tag) = &args.tag {
        payload.insert("tag".to_owned(), json!(tag));
    }

    let response = client
        .send(
            Method::POST,
            "/v1/push/notify",
            Some(&Value::Object(payload)),
        )
        .await?;

    let status = response.status();
    if status.is_success() {
        println!("sent");
        Ok(())
    } else {
        let body = response.text().await.unwrap_or_default();
        Err(ClientError::Status { status, body }.into())
    }
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
}
