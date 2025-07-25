use crate::{
    actors::woolworths::{self, WoolworthsActor},
    woolworths::{Woolworths, WoolworthsError},
};
use std::{collections::HashMap, time::Duration};
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, Level};

pub async fn woolworths_background(
    woolworths: &Woolworths,
    cancellation_token: CancellationToken,
) -> Result<(), WoolworthsError> {
    loop {
        let fut = async move {
            let actor = ractor::registry::where_is(WoolworthsActor::NAME.to_owned());
            if let Some(actor) = actor {
                let tracked_products = woolworths.get_all_tracked_products().await?;
                let mut tracked_map = HashMap::new();
                for product_group in tracked_products {
                    let response = woolworths.get_product(product_group.product_id).await;
                    match response {
                        Ok(resp) => {
                            tracked_map.insert(product_group, resp);
                        }
                        Err(e) => {
                            tracing::error!("error fetching: {e}")
                        }
                    }
                }

                actor.send_message(woolworths::WoolworthsMessage::TrackedProductGroup {
                    product_response_map: tracked_map,
                })?;
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
