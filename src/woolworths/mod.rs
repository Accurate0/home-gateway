use http::HeaderMap;
use sqlx::{Pool, Postgres};
use tracing::instrument;
use types::WoolworthsTrackedProduct;

use crate::{
    actors::woolworths::WoolworthsMessage, http::wrap_client_in_middleware,
    woolworths::types::WoolworthsProductResponse,
};

pub mod background;
pub mod types;

pub struct Woolworths {
    client: reqwest_middleware::ClientWithMiddleware,
    db: Pool<Postgres>,
}

#[derive(thiserror::Error, Debug)]
pub enum WoolworthsError {
    #[error(transparent)]
    HttpMiddleware(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("a actor message error occurred: {0}")]
    ActorMessage(#[from] ractor::MessagingErr<WoolworthsMessage>),
}

impl Woolworths {
    const BASE_URL: &str = "https://www.woolworths.com.au";
    const PRODUCT_SUFFIX: &str = "apis/ui/product/detail";

    pub fn new(db: Pool<Postgres>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::USER_AGENT,
            "Mozilla/5.0 (X11; Linux x86_64; rv:139.0) Gecko/20100101 Firefox/139.0"
                .parse()
                .unwrap(),
        );

        Self {
            db,
            client: wrap_client_in_middleware(
                reqwest::ClientBuilder::new()
                    .default_headers(headers)
                    .cookie_store(true)
                    .build()
                    .unwrap(),
            )
            .unwrap(),
        }
    }

    #[instrument(skip(self))]
    pub async fn get_all_tracked_products(
        &self,
    ) -> Result<Vec<WoolworthsTrackedProduct>, WoolworthsError> {
        sqlx::query_as!(
            WoolworthsTrackedProduct,
            "SELECT * FROM woolworths_product_tracking"
        )
        .fetch_all(&self.db)
        .await
        .map_err(WoolworthsError::from)
    }

    #[instrument(skip(self))]
    pub async fn get_product(
        &self,
        product_id: i64,
    ) -> Result<WoolworthsProductResponse, WoolworthsError> {
        self.client.get(Self::BASE_URL).send().await?;

        let product_url = format!("{}/{}/{}", Self::BASE_URL, Self::PRODUCT_SUFFIX, product_id);
        tracing::info!("fetching woolworths product: {product_id}");
        let resp = self
            .client
            .get(product_url)
            .send()
            .await?
            .error_for_status()?
            .json::<WoolworthsProductResponse>()
            .await?;

        Ok(resp)
    }
}
