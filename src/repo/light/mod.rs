pub mod types;

use chrono::TimeDelta;
use sqlx::{Pool, Postgres};

pub use types::{HistorySource, LightAttributes, LightSample, LightState, ProfileBucket};

#[derive(Clone)]
pub struct LightRepo {
    db: Pool<Postgres>,
}

impl LightRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.light.upsert_state_returning_previous", err)]
    pub async fn upsert_state_returning_previous(
        &self,
        ieee_addr: &str,
        attributes: &LightAttributes,
    ) -> Result<(LightState, Option<LightState>), sqlx::Error> {
        let row = sqlx::query!(
            "WITH prev AS ( \
                 SELECT state, brightness, colour_temp, colour \
                 FROM light_state WHERE ieee_address = $1 \
             ), upserted AS ( \
                 INSERT INTO light_state (ieee_address, state, brightness, colour_temp, colour) \
                 VALUES ($1, COALESCE($2, 'OFF'), $3, $4, $5) \
                 ON CONFLICT (ieee_address) DO UPDATE SET \
                     state = COALESCE(EXCLUDED.state, light_state.state), \
                     brightness = COALESCE(EXCLUDED.brightness, light_state.brightness), \
                     colour_temp = COALESCE(EXCLUDED.colour_temp, light_state.colour_temp), \
                     colour = COALESCE(EXCLUDED.colour, light_state.colour) \
                 RETURNING state, brightness, colour_temp, colour \
             ) \
             SELECT \
                 upserted.state AS \"state!\", \
                 upserted.brightness, \
                 upserted.colour_temp, \
                 upserted.colour, \
                 prev.state AS \"previous_state?\", \
                 prev.brightness AS \"previous_brightness?\", \
                 prev.colour_temp AS \"previous_colour_temp?\", \
                 prev.colour AS \"previous_colour?\" \
             FROM upserted LEFT JOIN prev ON true",
            ieee_addr,
            attributes.state.as_deref(),
            attributes.brightness,
            attributes.colour_temp,
            attributes.colour.as_deref(),
        )
        .fetch_one(&self.db)
        .await?;

        let state = LightState {
            on: row.state == "ON",
            brightness: row.brightness,
            colour_temp: row.colour_temp,
            colour: row.colour,
        };

        let previous = row.previous_state.map(|previous_state| LightState {
            on: previous_state == "ON",
            brightness: row.previous_brightness,
            colour_temp: row.previous_colour_temp,
            colour: row.previous_colour,
        });

        Ok((state, previous))
    }

    #[tracing::instrument(skip_all, name = "db.light.get", err)]
    pub async fn get(&self, ieee_addr: &str) -> Result<Option<LightState>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT state, brightness, colour_temp, colour FROM light_state WHERE ieee_address = $1",
            ieee_addr
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| LightState {
            on: row.state == "ON",
            brightness: row.brightness,
            colour_temp: row.colour_temp,
            colour: row.colour,
        }))
    }

    #[tracing::instrument(skip_all, name = "db.light.is_on", err)]
    pub async fn is_on(&self, ieee_addr: &str) -> Result<Option<bool>, sqlx::Error> {
        Ok(self.get(ieee_addr).await?.map(|state| state.on))
    }

    #[tracing::instrument(skip_all, name = "db.light.record_history", err)]
    pub async fn record_history(&self, sample: LightSample) -> Result<(), sqlx::Error> {
        self.record_history_many(&[sample]).await
    }

    #[tracing::instrument(skip_all, name = "db.light.record_history_many", err)]
    pub async fn record_history_many(&self, samples: &[LightSample]) -> Result<(), sqlx::Error> {
        let mut tx = self.db.begin().await?;

        for sample in samples {
            sqlx::query!(
                "INSERT INTO light_history \
                     (address, device_id, on_state, brightness, colour_temp, colour, source, event_id) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                sample.address,
                sample.device_id,
                sample.state.on,
                sample.state.brightness,
                sample.state.colour_temp,
                sample.state.colour,
                sample.source.as_str(),
                sample.event_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await
    }

    #[tracing::instrument(skip_all, name = "db.light.profile_all", err)]
    pub async fn profile_all(&self, window: TimeDelta) -> Result<Vec<ProfileBucket>, sqlx::Error> {
        sqlx::query_as!(
            ProfileBucket,
            r#"SELECT
                 address AS "address!",
                 EXTRACT(isodow FROM bucket AT TIME ZONE 'Australia/Perth')::smallint AS "isodow!",
                 (EXTRACT(hour FROM bucket AT TIME ZONE 'Australia/Perth')::smallint * 2
                    + EXTRACT(minute FROM bucket AT TIME ZONE 'Australia/Perth')::smallint / 30
                 )::smallint AS "slot!",
                 (sum(on_fraction * observations) / sum(observations))::double precision AS "on_fraction!",
                 sum(observations)::bigint AS "observations!",
                 sum(turned_on)::bigint AS "turned_on!"
               FROM light_activity_30m
               WHERE bucket >= now() - make_interval(secs => $1)
               GROUP BY 1, 2, 3
               ORDER BY 1, 2, 3"#,
            window.num_seconds() as f64,
        )
        .fetch_all(&self.db)
        .await
    }
}
