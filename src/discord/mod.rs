use anyhow::bail;
use interactions::remindme::RemindMeCommand;
use interactions::woolworths::WoolworthsCommand;
use sqlx::{Pool, Postgres};
use std::mem;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio_util::sync::CancellationToken;
use twilight_gateway::{
    CloseFrame, ConfigBuilder, Event, EventTypeFlags, Intents, Shard, StreamExt as _,
};
use twilight_http::Client;
use twilight_interactions::command::CreateCommand;
use twilight_model::application::interaction::{
    Interaction, InteractionData, application_command::CommandData,
};
use twilight_model::{
    application::interaction::InteractionContextType,
    gateway::{
        payload::outgoing::update_presence::UpdatePresencePayload,
        presence::{ActivityType, MinimalActivity, Status},
    },
    oauth::ApplicationIntegrationType,
};
use twilight_util::builder::command::CommandBuilder;

mod interactions;

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub async fn start_discord(
    token: String,
    pool: Pool<Postgres>,
    cancellation_token: CancellationToken,
) -> anyhow::Result<()> {
    let client = Arc::new(Client::new(token.clone()));
    let config = ConfigBuilder::new(token.clone(), Intents::empty())
        .presence(presence())
        .build();

    let application = client.current_user_application().await?.model().await?;

    let commands = [
        RemindMeCommand::create_command().into(),
        WoolworthsCommand::create_command().into(),
    ];
    let interaction_client = client.interaction(application.id);

    if let Err(error) = interaction_client.set_global_commands(&commands).await {
        tracing::error!(?error, "failed to register commands");
    }

    let global_commands = interaction_client.global_commands().await?.model().await?;

    let mut updated_commands = Vec::with_capacity(global_commands.len());
    for global_command in global_commands {
        let mut command = CommandBuilder::new(
            global_command.name,
            global_command.description,
            global_command.kind,
        )
        .integration_types(vec![
            ApplicationIntegrationType::GuildInstall,
            ApplicationIntegrationType::UserInstall,
        ])
        .contexts(vec![
            InteractionContextType::PrivateChannel,
            InteractionContextType::Guild,
        ]);

        for option in global_command.options {
            command = command.option(option);
        }

        updated_commands.push(command.build());
    }

    tracing::info!("updating commands: {}", updated_commands.len());
    interaction_client
        .set_global_commands(&updated_commands)
        .await?;
    tracing::info!("logged as {} with ID {}", application.name, application.id);

    // Start gateway shards.
    let shards =
        twilight_gateway::create_recommended(&client, config, |_id, builder| builder.build())
            .await?;
    let shard_len = shards.len();
    let mut senders = Vec::with_capacity(shard_len);
    let mut tasks = Vec::with_capacity(shard_len);

    for shard in shards {
        senders.push(shard.sender());
        tasks.push(tokio::spawn(runner(shard, client.clone(), pool.clone())));
    }

    cancellation_token.cancelled().await;
    SHUTDOWN.store(true, Ordering::Relaxed);
    for sender in senders {
        // Ignore error if shard's already shutdown.
        _ = sender.close(CloseFrame::NORMAL);
    }

    for jh in tasks {
        _ = jh.await;
    }

    Ok(())
}

async fn runner(mut shard: Shard, client: Arc<Client>, db: Pool<Postgres>) {
    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let event = match item {
            Ok(Event::GatewayClose(_)) if SHUTDOWN.load(Ordering::Relaxed) => break,
            Ok(event) => event,
            Err(error) => {
                tracing::warn!(?error, "error while receiving event");
                continue;
            }
        };

        tracing::info!(kind = ?event.kind(), shard = ?shard.id().number(), "received event");
        tokio::spawn(process_interactions(event, client.clone(), db.clone()));
    }
}

/// Process incoming interactions from Discord.
pub async fn process_interactions(event: Event, client: Arc<Client>, db: Pool<Postgres>) {
    // We only care about interaction events.
    let mut interaction = match event {
        Event::InteractionCreate(interaction) => interaction.0,
        _ => return,
    };

    // Extract the command data from the interaction.
    // We use mem::take to avoid cloning the data.
    let data = match mem::take(&mut interaction.data) {
        Some(InteractionData::ApplicationCommand(data)) => *data,
        _ => {
            tracing::warn!("ignoring non-command interaction");
            return;
        }
    };

    if let Err(error) = handle_command(interaction, data, &client, db.clone()).await {
        tracing::error!(?error, "error while handling command");
    }
}

/// Handle a command interaction.
async fn handle_command(
    interaction: Interaction,
    data: CommandData,
    client: &Client,
    db: Pool<Postgres>,
) -> anyhow::Result<()> {
    match &*data.name {
        "remindme" => RemindMeCommand::handle(interaction, data, client).await,
        "woolworths" => WoolworthsCommand::handle(interaction, data, client, db).await,
        name => bail!("unknown command: {}", name),
    }
}

fn presence() -> UpdatePresencePayload {
    let activity = MinimalActivity {
        kind: ActivityType::Watching,
        name: String::from("you"),
        url: None,
    };

    UpdatePresencePayload {
        activities: vec![activity.into()],
        afk: false,
        since: None,
        status: Status::Online,
    }
}
