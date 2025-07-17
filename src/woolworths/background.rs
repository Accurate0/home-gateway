use crate::{
    actors::woolworths::{self, WoolworthsActor},
    settings::WoolworthsSettings,
    woolworths::{Woolworths, WoolworthsError},
};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub async fn woolworths_background(
    woolworths: &Woolworths,
    woolworths_settings: &WoolworthsSettings,
    cancellation_token: CancellationToken,
) -> Result<(), WoolworthsError> {
    loop {
        let fut = async move {
            let actor = ractor::registry::where_is(WoolworthsActor::NAME.to_owned());
            if let Some(actor) = actor {
                let products = woolworths_settings.products.iter().map(|p| p.product_id);
                for product in products {
                    let response = woolworths.get_product(product).await?;
                    actor.send_message(woolworths::WoolworthsMessage::TrackedProduct(response))?;
                }
            } else {
                tracing::warn!("actor for woolworths not found");
            }

            tokio::time::sleep(Duration::from_secs(900)).await;
            Ok::<(), WoolworthsError>(())
        };
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                tracing::info!("cancellation requested");
                break Ok(());
            }

            result = fut => {
                if let Err(e) = result {
                    tracing::error!("error fetching from woolworths: {e}")
                }

            }
        }
    }
}
