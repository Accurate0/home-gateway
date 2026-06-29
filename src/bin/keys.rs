use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use home_gateway::api_types::{ApiKeyInfo, CreateKeyPayload, CreatedKey, UpdateKeyPayload};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "keys", about = "manage home-gateway api keys")]
struct Cli {
    #[arg(long, env = "HG_BASE_URL", default_value = "https://home.anurag.sh")]
    base_url: String,
    #[arg(long, env = "HG_API_KEY")]
    api_key: String,
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    Create {
        name: String,
        #[arg(value_delimiter = ',')]
        scopes: Vec<String>,
        #[arg(long)]
        expires_at: Option<DateTime<Utc>>,
    },
    Update {
        id: Uuid,
        #[arg(long)]
        name: Option<String>,
        #[arg(long, value_delimiter = ',')]
        scopes: Option<Vec<String>>,
        #[arg(long)]
        expires_at: Option<DateTime<Utc>>,
    },
    List,
    Revoke {
        id: Uuid,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = reqwest::Client::new();
    let base = cli.base_url.trim_end_matches('/');

    match cli.action {
        Action::Create {
            name,
            scopes,
            expires_at,
        } => {
            let payload = CreateKeyPayload {
                name,
                scopes,
                expires_at,
            };

            let created: CreatedKey = client
                .post(format!("{base}/v1/admin/keys"))
                .header("X-Api-Key", &cli.api_key)
                .json(&payload)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            println!("{}", serde_json::to_string_pretty(&created)?);
            println!("\nstore this key now, it will not be shown again");
        }
        Action::Update {
            id,
            name,
            scopes,
            expires_at,
        } => {
            let payload = UpdateKeyPayload {
                name,
                scopes,
                expires_at,
            };

            let resp = client
                .patch(format!("{base}/v1/admin/keys/{id}"))
                .header("X-Api-Key", &cli.api_key)
                .json(&payload)
                .send()
                .await?;

            if resp.status() == reqwest::StatusCode::NOT_FOUND {
                println!("no active key with id {id}");
            } else {
                let info: ApiKeyInfo = resp.error_for_status()?.json().await?;
                println!("{}", serde_json::to_string_pretty(&info)?);
            }
        }
        Action::List => {
            let keys: Vec<ApiKeyInfo> = client
                .get(format!("{base}/v1/admin/keys"))
                .header("X-Api-Key", &cli.api_key)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            println!("{}", serde_json::to_string_pretty(&keys)?);
        }
        Action::Revoke { id } => {
            let resp = client
                .delete(format!("{base}/v1/admin/keys/{id}"))
                .header("X-Api-Key", &cli.api_key)
                .send()
                .await?;

            if resp.status() == reqwest::StatusCode::NOT_FOUND {
                println!("no active key with id {id}");
            } else {
                resp.error_for_status()?;
                println!("revoked {id}");
            }
        }
    }

    Ok(())
}
