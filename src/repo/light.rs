use chrono::TimeDelta;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistorySource {
    Edge,
    Sample,
}

impl HistorySource {
    pub fn as_str(&self) -> &'static str {
        match self {
            HistorySource::Edge => "edge",
            HistorySource::Sample => "sample",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LightSample {
    pub address: String,
    pub device_id: Option<String>,
    pub source: HistorySource,
    pub event_id: Option<Uuid>,
    pub state: LightState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileBucket {
    pub address: String,
    pub isodow: i16,
    pub slot: i16,
    pub on_fraction: f64,
    pub observations: i64,
    pub turned_on: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightState {
    pub on: bool,
    pub brightness: Option<i32>,
    pub colour_temp: Option<i32>,
    pub colour: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LightAttributes {
    pub state: Option<String>,
    pub brightness: Option<i32>,
    pub colour_temp: Option<i32>,
    pub colour: Option<String>,
}

impl LightAttributes {
    pub fn state(state: impl Into<String>) -> Self {
        Self {
            state: Some(state.into()),
            ..Self::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.state.is_none()
            && self.brightness.is_none()
            && self.colour_temp.is_none()
            && self.colour.is_none()
    }
}

#[derive(Clone)]
pub struct LightRepo {
    db: Pool<Postgres>,
}

impl LightRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    pub async fn upsert_state(
        &self,
        ieee_addr: &str,
        attributes: &LightAttributes,
    ) -> Result<LightState, sqlx::Error> {
        let row = sqlx::query!(
            "INSERT INTO light_state (ieee_address, state, brightness, colour_temp, colour) \
             VALUES ($1, COALESCE($2, 'OFF'), $3, $4, $5) \
             ON CONFLICT (ieee_address) DO UPDATE SET \
                 state = COALESCE(EXCLUDED.state, light_state.state), \
                 brightness = COALESCE(EXCLUDED.brightness, light_state.brightness), \
                 colour_temp = COALESCE(EXCLUDED.colour_temp, light_state.colour_temp), \
                 colour = COALESCE(EXCLUDED.colour, light_state.colour) \
             RETURNING state, brightness, colour_temp, colour",
            ieee_addr,
            attributes.state.as_deref(),
            attributes.brightness,
            attributes.colour_temp,
            attributes.colour.as_deref(),
        )
        .fetch_one(&self.db)
        .await?;

        Ok(LightState {
            on: row.state == "ON",
            brightness: row.brightness,
            colour_temp: row.colour_temp,
            colour: row.colour,
        })
    }

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

    pub async fn is_on(&self, ieee_addr: &str) -> Result<Option<bool>, sqlx::Error> {
        Ok(self.get(ieee_addr).await?.map(|state| state.on))
    }

    pub async fn record_history(&self, sample: LightSample) -> Result<(), sqlx::Error> {
        self.record_history_many(&[sample]).await
    }

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

    pub async fn profile(
        &self,
        address: &str,
        window: TimeDelta,
    ) -> Result<Vec<ProfileBucket>, sqlx::Error> {
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
               WHERE address = $1 AND bucket >= now() - make_interval(secs => $2)
               GROUP BY 1, 2, 3
               ORDER BY 1, 2, 3"#,
            address,
            window.num_seconds() as f64,
        )
        .fetch_all(&self.db)
        .await
    }

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
