use crate::integrations::woolworths::types::WoolworthsTrackedProduct;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;

#[derive(Clone)]
pub struct WoolworthsRepo {
    db: Pool<Postgres>,
}

pub struct WoolworthsProductRow {
    pub display_name: String,
    pub price: f64,
}

pub struct WoolworthsPriceHistoryRow {
    pub product_id: i64,
    pub display_name: String,
    pub price: f64,
    pub time: chrono::DateTime<chrono::Utc>,
}

impl WoolworthsRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn prices(&self) -> Result<HashMap<i64, f64>, sqlx::Error> {
        let rows = sqlx::query!("SELECT product_id, price FROM woolworths_product_price")
            .fetch_all(&self.db)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| (row.product_id, row.price))
            .collect())
    }

    pub async fn upsert_price(
        &self,
        product_id: i64,
        price: f64,
        display_name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO woolworths_product_price(product_id, price, display_name) VALUES ($1, $2, $3)
                        ON CONFLICT(product_id)
                        DO UPDATE SET price = EXCLUDED.price, display_name = EXCLUDED.display_name"#,
            product_id,
            price,
            display_name
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn append_price_history(
        &self,
        product_id: i64,
        price: f64,
        display_name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO woolworths_price_history(product_id, price, display_name) VALUES ($1, $2, $3)"#,
            product_id,
            price,
            display_name
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
    pub async fn products(&self) -> Result<Vec<WoolworthsProductRow>, sqlx::Error> {
        sqlx::query_as!(
            WoolworthsProductRow,
            r#"SELECT display_name, price FROM woolworths_product_price"#,
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn price_history(
        &self,
        product_id: i64,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<WoolworthsPriceHistoryRow>, sqlx::Error> {
        sqlx::query_as!(
            WoolworthsPriceHistoryRow,
            r#"SELECT product_id, display_name, price, "time"
               FROM woolworths_price_history
               WHERE product_id = $1 AND "time" >= $2
               ORDER BY "time" ASC"#,
            product_id,
            since
        )
        .fetch_all(&self.db)
        .await
    }
    pub async fn tracked_products(&self) -> Result<Vec<WoolworthsTrackedProduct>, sqlx::Error> {
        sqlx::query_as!(
            WoolworthsTrackedProduct,
            "SELECT id, product_id FROM woolworths_product_tracking"
        )
        .fetch_all(&self.db)
        .await
    }
}
