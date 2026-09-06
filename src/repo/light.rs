use sqlx::{Pool, Postgres};

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
}
