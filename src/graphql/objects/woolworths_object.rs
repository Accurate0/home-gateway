use crate::repo::RepoRegistry;
use async_graphql::{InputObject, Object, SimpleObject};
use chrono::{DateTime, Utc};
use itertools::Itertools;

pub struct WoolworthsObject {}

#[derive(SimpleObject)]
pub struct WoolworthsProducts {
    pub name: String,
    pub price: f64,
}

#[derive(InputObject)]
pub struct WoolworthsPriceHistoryInput {
    pub product_id: i64,
    pub since: DateTime<Utc>,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct WoolworthsPricePoint {
    pub product_id: i64,
    pub name: String,
    pub price: f64,
    pub time: DateTime<Utc>,
}

#[Object]
impl WoolworthsObject {
    pub async fn products(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<WoolworthsProducts>> {
        let repos = ctx.data::<RepoRegistry>()?;

        Ok(repos
            .woolworths()
            .products()
            .await?
            .into_iter()
            .map(|r| WoolworthsProducts {
                name: r.display_name,
                price: r.price,
            })
            .collect_vec())
    }

    pub async fn price_history(
        &self,
        ctx: &async_graphql::Context<'_>,
        input: WoolworthsPriceHistoryInput,
    ) -> async_graphql::Result<Vec<WoolworthsPricePoint>> {
        let repos = ctx.data::<RepoRegistry>()?;

        Ok(repos
            .woolworths()
            .price_history(input.product_id, input.since)
            .await?
            .into_iter()
            .map(|r| WoolworthsPricePoint {
                product_id: r.product_id,
                name: r.display_name,
                price: r.price,
                time: r.time,
            })
            .collect_vec())
    }
}
