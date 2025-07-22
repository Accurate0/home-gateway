use crate::{
    actors::woolworths::{self, WoolworthsActor},
    settings::WoolworthsSettings,
    woolworths::{Woolworths, WoolworthsError},
};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, Level};

pub async fn woolworths_background(
    woolworths: &Woolworths,
    woolworths_settings: &WoolworthsSettings,
    cancellation_token: CancellationToken,
) -> Result<(), WoolworthsError> {
    loop {
        let fut = async move {
            let actor = ractor::registry::where_is(WoolworthsActor::NAME.to_owned());
            if let Some(actor) = actor {
                for product_group in &woolworths_settings.products {
                    let products = &product_group.product_ids;
                    let mut product_group_responses = Vec::with_capacity(products.len());
                    for product in products {
                        let response = woolworths.get_product(*product).await?;
                        product_group_responses.push(response);
                    }

                    actor.send_message(woolworths::WoolworthsMessage::TrackedProductGroup {
                        id: product_group.id.to_owned(),
                        product_responses: product_group_responses,
                    })?;
                }
            } else {
                tracing::warn!("actor for woolworths not found");
            }

            Ok::<(), WoolworthsError>(())
        }
        .instrument(tracing::span!(
            Level::INFO,
            "woolworths_background",
            "otel.name" = "woolworths_background"
        ));

        tokio::select! {
            _ = cancellation_token.cancelled() => {
                tracing::info!("cancellation requested");
                break Ok(());
            }

            result = fut => {
                if let Err(e) = result {
                    tracing::error!("error fetching from woolworths: {e}")
                }

                tokio::time::sleep(Duration::from_secs(900)).await;
            }
        }
    }
}
