use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct PushRepo {
    db: Pool<Postgres>,
}

impl PushRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn tokens(&self) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar!("SELECT token FROM android_push_tokens")
            .fetch_all(&self.db)
            .await
    }

    pub async fn upsert_token(&self, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO android_push_tokens (token) VALUES ($1) ON CONFLICT (token) DO UPDATE SET updated_at = now()",
            token
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn delete_token(&self, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM android_push_tokens WHERE token = $1", token)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}
