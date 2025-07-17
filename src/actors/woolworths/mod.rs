use std::collections::HashMap;

use crate::{
    notify::notify, settings::WoolworthsSettings, types::SharedActorState,
    woolworths::types::WoolworthsProductResponse,
};
use ractor::Actor;

pub enum WoolworthsMessage {
    TrackedProduct(WoolworthsProductResponse),
}

pub struct WoolworthsActorState {
    pub woolworths_product_price: HashMap<i64, f64>,
}

pub struct WoolworthsActor {
    pub settings: WoolworthsSettings,
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

    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            WoolworthsMessage::TrackedProduct(woolworths_product_response) => {
                let product_id = woolworths_product_response.product.stockcode;
                let last_price = state
                    .woolworths_product_price
                    .entry(product_id)
                    .or_insert(woolworths_product_response.product.price);

                if let Some(product) = self
                    .settings
                    .products
                    .iter()
                    .find(|p| p.product_id == woolworths_product_response.product.stockcode)
                {
                    let current_price = woolworths_product_response.product.price;
                    let is_price_lower = current_price < *last_price;
                    if is_price_lower {
                        let message = format!(
                            "{} price is now ${} at Woolworths",
                            product.name, current_price
                        );
                        notify(&product.notify, message);
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
