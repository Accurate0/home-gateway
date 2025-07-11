use crate::{
    actors::reminder::{ReminderActor, ReminderActorMessage},
    timedelta_format::parse_datetime_str,
};
use anyhow::Context;
use twilight_http::Client;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::{Interaction, application_command::CommandData},
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};

#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "remindme", desc = "Remind me about something later")]
pub struct RemindMeCommand {
    /// How long till reminder e.g. 5m, 1h
    time: String,
    /// What to remind about
    message: String,
}

impl RemindMeCommand {
    pub async fn handle(
        interaction: Interaction,
        data: CommandData,
        client: &Client,
    ) -> anyhow::Result<()> {
        let interaction_client = client.interaction(interaction.application_id);

        interaction_client
            .create_response(
                interaction.id,
                &interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::DeferredChannelMessageWithSource,
                    data: Some(InteractionResponseData {
                        flags: Some(MessageFlags::EPHEMERAL),
                        ..Default::default()
                    }),
                },
            )
            .await?;

        let command = RemindMeCommand::from_interaction(data.into())
            .context("failed to parse command data")?;

        let actor = ractor::registry::where_is(ReminderActor::NAME.to_owned());

        if let Some(actor) = actor {
            let time = parse_datetime_str(&command.time);
            if let Err(_) = time {
                interaction_client
                    .update_response(&interaction.token)
                    .content(Some("invalid time format, e.g. 5m 1h"))
                    .await?;
                return Ok(());
            }

            let time = time.unwrap().to_std()?;
            actor.send_message(ReminderActorMessage::SetReminder {
                message: command.message,
                delay: time,
                channel_id: interaction.channel.as_ref().unwrap().id.get(),
                user_id: interaction.author_id().unwrap().get(),
            })?;

            interaction_client
                .update_response(&interaction.token)
                .content(Some("Reminder set"))
                .await?;

            Ok(())
        } else {
            interaction_client
                .update_response(&interaction.token)
                .content(Some("cannot set reminder right now"))
                .await?;

            Ok(())
        }
    }
}
