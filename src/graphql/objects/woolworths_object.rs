use async_graphql::{Object, SimpleObject};
use itertools::Itertools;
use sqlx::{Pool, Postgres};

pub struct WoolworthsObject {}

#[derive(SimpleObject)]
pub struct WoolworthsProducts {
    pub name: String,
    pub price: f64,
}

#[Object]
impl WoolworthsObject {
    pub async fn products(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<WoolworthsProducts>> {
        let db = ctx.data::<Pool<Postgres>>()?;

        Ok(
            sqlx::query!(r#"SELECT display_name, price FROM woolworths_product_price"#,)
                .fetch_all(db)
                .await?
                .into_iter()
                .map(|r| WoolworthsProducts {
                    name: r.display_name,
                    price: r.price,
                })
                .collect_vec(),
        )
    }
}
