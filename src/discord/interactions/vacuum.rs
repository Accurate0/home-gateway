use crate::actors::vacuum::{VacuumActor, VacuumMessage};
use anyhow::Context;
use twilight_http::Client;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::{Interaction, application_command::CommandData},
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};

#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "vacuum", desc = "Control the vacuum")]
pub struct VacuumCommand {
    /// start, stop
    action: String,
}

impl VacuumCommand {
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

        let command =
            VacuumCommand::from_interaction(data.into()).context("failed to parse command data")?;

        let actor = ractor::registry::where_is(VacuumActor::NAME.to_owned());

        if let Some(actor) = actor {
            match command.action.as_str() {
                "start" => {
                    actor.send_message(VacuumMessage::Start)?;

                    interaction_client
                        .update_response(&interaction.token)
                        .content(Some("vacuum started"))
                        .await?;
                }
                "stop" => {
                    actor.send_message(VacuumMessage::Home)?;

                    interaction_client
                        .update_response(&interaction.token)
                        .content(Some("vacuum going home"))
                        .await?;
                }
                _ => {
                    interaction_client
                        .update_response(&interaction.token)
                        .content(Some("invalid action, must be start or stop"))
                        .await?;
                }
            };

            Ok(())
        } else {
            interaction_client
                .update_response(&interaction.token)
                .content(Some("cannot control vacuum right now"))
                .await?;

            Ok(())
        }
    }
}
