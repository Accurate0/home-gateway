use crate::{
    notify::notify, settings::WoolworthsSettings, woolworths::types::WoolworthsProductResponse,
};
use ractor::Actor;

pub enum WoolworthsMessage {
    TrackedProduct(WoolworthsProductResponse),
}

pub struct WoolworthsActor {
    pub settings: WoolworthsSettings,
}

impl WoolworthsActor {
    pub const NAME: &str = "woolworths";
}

impl Actor for WoolworthsActor {
    type Msg = WoolworthsMessage;
    type State = ();
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        _state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            WoolworthsMessage::TrackedProduct(woolworths_product_response) => {
                if let Some(product) = self
                    .settings
                    .products
                    .iter()
                    .find(|p| p.product_id == woolworths_product_response.product.stockcode)
                {
                    let current_price = woolworths_product_response.product.price;
                    let is_price_lower = current_price < last_price;
                    if is_price_lower {
                        let message = format!("{} price is now {}", product.name, current_price);
                        notify(&product.notify, message);
                    }
                }
            }
        }
        Ok(())
    }
}
