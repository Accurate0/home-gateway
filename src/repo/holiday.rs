use chrono::NaiveDate;
use sqlx::{Pool, Postgres};

use crate::integrations::holidays::Holiday;

#[derive(Clone)]
pub struct HolidayRepo {
    db: Pool<Postgres>,
}

impl HolidayRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.holiday.replace", fields(holidays = holidays.len()), err)]
    pub async fn replace(&self, holidays: &[Holiday]) -> Result<u64, sqlx::Error> {
        let mut tx = self.db.begin().await?;

        let uids: Vec<String> = holidays.iter().map(|holiday| holiday.uid.clone()).collect();

        sqlx::query!("DELETE FROM public_holiday WHERE uid <> ALL($1)", &uids)
            .execute(&mut *tx)
            .await?;

        let mut stored = 0;

        for holiday in holidays {
            stored += sqlx::query!(
                r#"
                INSERT INTO public_holiday (uid, date, name, public, regions)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (uid) DO UPDATE SET
                    date = EXCLUDED.date,
                    name = EXCLUDED.name,
                    public = EXCLUDED.public,
                    regions = EXCLUDED.regions,
                    updated_at = now()
                "#,
                holiday.uid,
                holiday.date,
                holiday.name,
                holiday.public,
                &holiday.regions,
            )
            .execute(&mut *tx)
            .await?
            .rows_affected();
        }

        tx.commit().await?;

        Ok(stored)
    }

    #[tracing::instrument(skip_all, name = "db.holiday.between", err)]
    pub async fn between(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Holiday>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT uid, date, name, public, regions
            FROM public_holiday
            WHERE date BETWEEN $1 AND $2
            ORDER BY date ASC
            "#,
            from,
            to,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Holiday {
                uid: row.uid,
                date: row.date,
                name: row.name,
                public: row.public,
                regions: row.regions,
            })
            .collect())
    }
}
