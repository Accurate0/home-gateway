use anyhow::Context;
use lazy_static::lazy_static;
use regex::Regex;
use sqlx::{Pool, Postgres};
use twilight_http::Client;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::{Interaction, application_command::CommandData},
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};

lazy_static! {
    static ref PRODUCT_ID_REGEX: Regex = Regex::new(r#"\/(\d+)\/"#).unwrap();
}

#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "woolworths", desc = "Tell me if this product is on sale")]
pub struct WoolworthsCommand {
    /// Link to the product page to track
    link: String,
}

impl WoolworthsCommand {
    pub async fn handle(
        interaction: Interaction,
        data: CommandData,
        client: &Client,
        db: Pool<Postgres>,
    ) -> anyhow::Result<()> {
        let interaction_client = client.interaction(interaction.application_id);

        interaction_client
            .create_response(
                interaction.id,
                &interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::DeferredChannelMessageWithSource,
                    data: Some(InteractionResponseData {
                        flags: None,
                        ..Default::default()
                    }),
                },
            )
            .await?;

        let command = WoolworthsCommand::from_interaction(data.into())
            .context("failed to parse command data")?;

        let product_id = PRODUCT_ID_REGEX.captures(&command.link);
        if product_id.is_none() {
            tracing::error!("failed to find product id using regex");
            interaction_client
                .update_response(&interaction.token)
                .content(Some("could not get product id from link"))
                .await?;
            return Ok(());
        }

        let product_id = product_id.unwrap();
        tracing::error!("trying product id: {product_id:?}");
        let product_id = product_id
            .get(1)
            .and_then(|p| p.as_str().parse::<i64>().ok());

        if product_id.is_none() {
            tracing::error!("failed to parse as integer");
            interaction_client
                .update_response(&interaction.token)
                .content(Some("could not get product id from link"))
                .await?;
            return Ok(());
        }

        let product_id = product_id.unwrap();
        let interaction_author_vec = vec![interaction.author_id().unwrap().get() as i64];
        let interaction_author_id = interaction_author_vec.as_slice();
        let channel_id = interaction.channel.unwrap().id.get() as i64;

        sqlx::query!(
            "INSERT INTO woolworths_product_tracking (product_id, notify_channel, mentions) VALUES ($1, $2, $3)",
            product_id,
            channel_id,
            interaction_author_id
        )
        .execute(&db)
        .await?;

        interaction_client
            .update_response(&interaction.token)
            .content(Some("added product for tracking"))
            .await?;

        Ok(())
    }
}
