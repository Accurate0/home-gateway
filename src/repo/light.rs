use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct LightRepo {
    db: Pool<Postgres>,
}

impl LightRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn upsert_state(&self, ieee_addr: &str, state: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO light_state (ieee_address, state) VALUES ($1, $2) ON CONFLICT (ieee_address) DO UPDATE SET state = EXCLUDED.state",
            ieee_addr,
            state,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn is_on(&self, ieee_addr: &str) -> Result<Option<bool>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT state FROM light_state WHERE ieee_address = $1",
            ieee_addr
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| row.state == "ON"))
    }
}
