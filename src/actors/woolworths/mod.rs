use crate::{
    notify::notify,
    settings::NotifySource,
    types::SharedActorState,
    woolworths::types::{WoolworthsProductResponse, WoolworthsTrackedProduct},
};
use ractor::Actor;
use std::collections::HashMap;

pub enum WoolworthsMessage {
    TrackedProductGroup {
        product_response_map: HashMap<WoolworthsTrackedProduct, WoolworthsProductResponse>,
    },
}

pub struct WoolworthsActorState {
    pub woolworths_product_price: HashMap<i64, f64>,
}

pub struct WoolworthsActor {
    pub shared_actor_state: SharedActorState,
}

impl WoolworthsActor {
    pub const NAME: &str = "woolworths";
}

impl Actor for WoolworthsActor {
    type Msg = WoolworthsMessage;
    type State = WoolworthsActorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let results = sqlx::query!("SELECT product_id, price FROM woolworths_product_price")
            .fetch_all(&self.shared_actor_state.db)
            .await?;

        let mut price_map = HashMap::new();
        for result in results {
            price_map.insert(result.product_id, result.price);
        }

        Ok(WoolworthsActorState {
            woolworths_product_price: price_map,
        })
    }

    #[tracing::instrument(name = "woolworths-actor", skip(self, _myself, message, state))]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            WoolworthsMessage::TrackedProductGroup {
                product_response_map,
            } => {
                for (tracked, response) in product_response_map {
                    let product_id = response.product.stockcode;
                    let last_price = state
                        .woolworths_product_price
                        .entry(product_id)
                        .or_insert(response.product.price);

                    let current_price = response.product.price;
                    let is_price_lower = current_price < *last_price;
                    if is_price_lower {
                        let price_down_by = *last_price - current_price;
                        let product_string = format!(
                            "{} - ${} (-${:.2})",
                            response.product.display_name, response.product.price, price_down_by
                        );

                        let notify_source = NotifySource::Discord {
                            channel_id: tracked.notify_channel as u64,
                            mentions: tracked.mentions.iter().map(|m| *m as u64).collect(),
                        };

                        notify(&[notify_source], product_string, true);
                    }

                    state
                        .woolworths_product_price
                        .entry(product_id)
                        .and_modify(|price| *price = current_price);

                    sqlx::query!(
                        r#"INSERT INTO woolworths_product_price(product_id, price) VALUES ($1, $2)
                        ON CONFLICT(product_id)
                        DO UPDATE SET price = EXCLUDED.price"#,
                        product_id,
                        current_price
                    )
                    .execute(&self.shared_actor_state.db)
                    .await?;
                }
            }
        }
        Ok(())
    }
}
