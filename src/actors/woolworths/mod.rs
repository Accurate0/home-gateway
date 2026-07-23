use crate::{
    event_bus::EventBusMessage,
    types::SharedActorState,
    woolworths::{
        Woolworths,
        types::{WoolworthsProductResponse, WoolworthsTrackedProduct},
    },
};
use ractor::Actor;
use std::{collections::HashMap, time::Duration};
use uuid::Uuid;

pub enum WoolworthsMessage {
    TrackedProductGroup {
        product_response_map: HashMap<WoolworthsTrackedProduct, WoolworthsProductResponse>,
    },
    CheckProductPrices,
}

pub struct WoolworthsActorState {
    pub woolworths_product_price: HashMap<i64, f64>,
}

pub struct WoolworthsActor {
    pub shared_actor_state: SharedActorState,
    pub woolworths: Woolworths,
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
        myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let results = sqlx::query!("SELECT product_id, price FROM woolworths_product_price")
            .fetch_all(&self.shared_actor_state.db)
            .await?;

        let mut price_map = HashMap::new();
        for result in results {
            price_map.insert(result.product_id, result.price);
        }

        let refresh = self
            .shared_actor_state
            .settings
            .woolworths
            .refresh
            .to_std()
            .unwrap_or(Duration::from_secs(3600));
        myself.send_interval(refresh, || WoolworthsMessage::CheckProductPrices);

        Ok(WoolworthsActorState {
            woolworths_product_price: price_map,
        })
    }

    async fn handle(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            WoolworthsMessage::CheckProductPrices => {
                tracing::info!("checking woolworths prices");
                let tracked_products = self.woolworths.get_all_tracked_products().await?;
                let mut tracked_map = HashMap::new();
                for product_group in tracked_products {
                    let response = self.woolworths.get_product(product_group.product_id).await;
                    match response {
                        Ok(resp) => {
                            tracked_map.insert(product_group, resp);
                        }
                        Err(e) => {
                            tracing::error!("error fetching: {e}")
                        }
                    }
                }

                myself.send_message(WoolworthsMessage::TrackedProductGroup {
                    product_response_map: tracked_map,
                })?;
            }
            WoolworthsMessage::TrackedProductGroup {
                product_response_map,
            } => {
                for (_tracked, response) in product_response_map {
                    let product_id = response.product.stockcode;
                    let product_name = response.product.display_name;
                    let last_price = state
                        .woolworths_product_price
                        .entry(product_id)
                        .or_insert(response.product.price);

                    let current_price = response.product.price;
                    let is_price_lower = current_price < *last_price;
                    if is_price_lower {
                        self.shared_actor_state
                            .event_bus
                            .publish(EventBusMessage::Woolworths {
                                event_id: Uuid::new_v4(),
                                product_id,
                                name: product_name.clone(),
                                old_price: *last_price,
                                new_price: current_price,
                            });
                    }

                    state
                        .woolworths_product_price
                        .entry(product_id)
                        .and_modify(|price| *price = current_price);

                    sqlx::query!(
                        r#"INSERT INTO woolworths_product_price(product_id, price, display_name) VALUES ($1, $2, $3)
                        ON CONFLICT(product_id)
                        DO UPDATE SET price = EXCLUDED.price, display_name = EXCLUDED.display_name"#,
                        product_id,
                        current_price,
                        product_name
                    )
                    .execute(&self.shared_actor_state.db)
                    .await?;

                    sqlx::query!(
                        r#"INSERT INTO woolworths_price_history(product_id, price, display_name) VALUES ($1, $2, $3)"#,
                        product_id,
                        current_price,
                        product_name
                    )
                    .execute(&self.shared_actor_state.db)
                    .await?;
                }
            }
        }
        Ok(())
    }
}
