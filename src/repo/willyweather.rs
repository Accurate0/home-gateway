use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use sqlx::{Pool, Postgres, Transaction};

use crate::integrations::willyweather::types::{Forecast, ForecastDetails, ForecastHour};
use crate::integrations::willyweather::{FORECAST_OFFSET, precis_emoji};

#[derive(thiserror::Error, Debug)]
pub enum WillyWeatherRepoError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("could not parse stored forecast timestamp: {0}")]
    Timestamp(#[from] chrono::ParseError),
}

#[derive(Clone)]
pub struct WillyWeatherRepo {
    db: Pool<Postgres>,
}

struct DayColumns {
    location: Vec<String>,
    date: Vec<NaiveDate>,
    precis_code: Vec<String>,
    precis: Vec<String>,
    min_temp: Vec<i32>,
    max_temp: Vec<i32>,
    uv_max: Vec<Option<f64>>,
    rain_probability: Vec<Option<i32>>,
    rain_start_range: Vec<Option<i32>>,
    rain_end_range: Vec<Option<i32>>,
    rain_range_code: Vec<Option<String>>,
    wind_max_speed: Vec<Option<f64>>,
    first_light: Vec<Option<DateTime<Utc>>>,
    sunrise: Vec<Option<DateTime<Utc>>>,
    sunset: Vec<Option<DateTime<Utc>>>,
    last_light: Vec<Option<DateTime<Utc>>>,
}

struct HourColumns {
    location: Vec<String>,
    time: Vec<DateTime<Utc>>,
    temperature: Vec<Option<f64>>,
    wind_speed: Vec<Option<f64>>,
    wind_direction: Vec<Option<f64>>,
    wind_direction_text: Vec<Option<String>>,
}

fn timestamp(value: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    Ok(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc))
}

fn optional_timestamp(value: Option<&str>) -> Result<Option<DateTime<Utc>>, chrono::ParseError> {
    value.map(timestamp).transpose()
}

fn local_string(value: DateTime<Utc>) -> String {
    value.with_timezone(&FORECAST_OFFSET).to_rfc3339()
}

fn local_midnight(date: NaiveDate) -> String {
    FORECAST_OFFSET
        .from_local_datetime(&date.and_time(NaiveTime::MIN))
        .single()
        .map_or_else(|| date.to_string(), |midnight| midnight.to_rfc3339())
}

impl DayColumns {
    fn new(location: &str, days: &[ForecastDetails]) -> Result<Self, chrono::ParseError> {
        let mut columns = Self {
            location: Vec::with_capacity(days.len()),
            date: Vec::with_capacity(days.len()),
            precis_code: Vec::with_capacity(days.len()),
            precis: Vec::with_capacity(days.len()),
            min_temp: Vec::with_capacity(days.len()),
            max_temp: Vec::with_capacity(days.len()),
            uv_max: Vec::with_capacity(days.len()),
            rain_probability: Vec::with_capacity(days.len()),
            rain_start_range: Vec::with_capacity(days.len()),
            rain_end_range: Vec::with_capacity(days.len()),
            rain_range_code: Vec::with_capacity(days.len()),
            wind_max_speed: Vec::with_capacity(days.len()),
            first_light: Vec::with_capacity(days.len()),
            sunrise: Vec::with_capacity(days.len()),
            sunset: Vec::with_capacity(days.len()),
            last_light: Vec::with_capacity(days.len()),
        };

        for day in days {
            let date = DateTime::parse_from_rfc3339(&day.date_time)?.date_naive();

            columns.location.push(location.to_owned());
            columns.date.push(date);
            columns.precis_code.push(day.code.clone());
            columns.precis.push(day.description.clone());
            columns.min_temp.push(day.min as i32);
            columns.max_temp.push(day.max as i32);
            columns.uv_max.push(day.uv);
            columns
                .rain_probability
                .push(day.rain_probability.map(|value| value as i32));
            columns
                .rain_start_range
                .push(day.rain_start_range.map(|value| value as i32));
            columns
                .rain_end_range
                .push(day.rain_end_range.map(|value| value as i32));
            columns.rain_range_code.push(day.rain_range_code.clone());
            columns.wind_max_speed.push(day.wind_max_speed);
            columns
                .first_light
                .push(optional_timestamp(day.first_light.as_deref())?);
            columns
                .sunrise
                .push(optional_timestamp(day.sunrise.as_deref())?);
            columns
                .sunset
                .push(optional_timestamp(day.sunset.as_deref())?);
            columns
                .last_light
                .push(optional_timestamp(day.last_light.as_deref())?);
        }

        Ok(columns)
    }
}

impl HourColumns {
    fn new(location: &str, hours: &[ForecastHour]) -> Result<Self, chrono::ParseError> {
        let mut columns = Self {
            location: Vec::with_capacity(hours.len()),
            time: Vec::with_capacity(hours.len()),
            temperature: Vec::with_capacity(hours.len()),
            wind_speed: Vec::with_capacity(hours.len()),
            wind_direction: Vec::with_capacity(hours.len()),
            wind_direction_text: Vec::with_capacity(hours.len()),
        };

        for hour in hours {
            columns.location.push(location.to_owned());
            columns.time.push(timestamp(&hour.date_time)?);
            columns.temperature.push(hour.temperature);
            columns.wind_speed.push(hour.wind_speed);
            columns.wind_direction.push(hour.wind_direction);
            columns
                .wind_direction_text
                .push(hour.wind_direction_text.clone());
        }

        Ok(columns)
    }
}

impl WillyWeatherRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(
        skip_all,
        name = "db.willyweather.replace_forecast",
        fields(location),
        err
    )]
    pub async fn replace_forecast(
        &self,
        tx: &mut Transaction<'static, Postgres>,
        location: &str,
        forecast: &Forecast,
    ) -> Result<u64, WillyWeatherRepoError> {
        let days = DayColumns::new(location, &forecast.days)?;
        let hours = HourColumns::new(location, &forecast.hours)?;

        sqlx::query!(
            "DELETE FROM willyweather_forecast_day WHERE location = $1",
            location
        )
        .execute(&mut **tx)
        .await?;

        sqlx::query!(
            "DELETE FROM willyweather_forecast_hour WHERE location = $1",
            location
        )
        .execute(&mut **tx)
        .await?;

        let inserted_days = sqlx::query!(
            r#"
            INSERT INTO willyweather_forecast_day (
                location, date, precis_code, precis, min_temp, max_temp, uv_max,
                rain_probability, rain_start_range, rain_end_range, rain_range_code,
                wind_max_speed, first_light, sunrise, sunset, last_light
            )
            SELECT * FROM UNNEST(
                $1::TEXT[], $2::DATE[], $3::TEXT[], $4::TEXT[], $5::INTEGER[], $6::INTEGER[],
                $7::DOUBLE PRECISION[], $8::INTEGER[], $9::INTEGER[], $10::INTEGER[], $11::TEXT[],
                $12::DOUBLE PRECISION[], $13::TIMESTAMPTZ[], $14::TIMESTAMPTZ[],
                $15::TIMESTAMPTZ[], $16::TIMESTAMPTZ[]
            )
            "#,
            &days.location,
            &days.date,
            &days.precis_code,
            &days.precis,
            &days.min_temp,
            &days.max_temp,
            &days.uv_max as &[Option<f64>],
            &days.rain_probability as &[Option<i32>],
            &days.rain_start_range as &[Option<i32>],
            &days.rain_end_range as &[Option<i32>],
            &days.rain_range_code as &[Option<String>],
            &days.wind_max_speed as &[Option<f64>],
            &days.first_light as &[Option<DateTime<Utc>>],
            &days.sunrise as &[Option<DateTime<Utc>>],
            &days.sunset as &[Option<DateTime<Utc>>],
            &days.last_light as &[Option<DateTime<Utc>>],
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();

        let inserted_hours = sqlx::query!(
            r#"
            INSERT INTO willyweather_forecast_hour (
                location, "time", temperature, wind_speed, wind_direction, wind_direction_text
            )
            SELECT * FROM UNNEST(
                $1::TEXT[], $2::TIMESTAMPTZ[], $3::DOUBLE PRECISION[],
                $4::DOUBLE PRECISION[], $5::DOUBLE PRECISION[], $6::TEXT[]
            )
            "#,
            &hours.location,
            &hours.time,
            &hours.temperature as &[Option<f64>],
            &hours.wind_speed as &[Option<f64>],
            &hours.wind_direction as &[Option<f64>],
            &hours.wind_direction_text as &[Option<String>],
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();

        Ok(inserted_days + inserted_hours)
    }

    #[tracing::instrument(
        skip_all,
        name = "db.willyweather.append_history",
        fields(location),
        err
    )]
    pub async fn append_history(
        &self,
        tx: &mut Transaction<'static, Postgres>,
        location: &str,
        forecast: &Forecast,
    ) -> Result<u64, WillyWeatherRepoError> {
        let days = DayColumns::new(location, &forecast.days)?;
        let hours = HourColumns::new(location, &forecast.hours)?;

        let inserted_days = sqlx::query!(
            r#"
            INSERT INTO willyweather_forecast_day_history (
                location, date, precis_code, precis, min_temp, max_temp, uv_max,
                rain_probability, rain_start_range, rain_end_range, rain_range_code,
                wind_max_speed, first_light, sunrise, sunset, last_light
            )
            SELECT * FROM UNNEST(
                $1::TEXT[], $2::DATE[], $3::TEXT[], $4::TEXT[], $5::INTEGER[], $6::INTEGER[],
                $7::DOUBLE PRECISION[], $8::INTEGER[], $9::INTEGER[], $10::INTEGER[], $11::TEXT[],
                $12::DOUBLE PRECISION[], $13::TIMESTAMPTZ[], $14::TIMESTAMPTZ[],
                $15::TIMESTAMPTZ[], $16::TIMESTAMPTZ[]
            )
            "#,
            &days.location,
            &days.date,
            &days.precis_code,
            &days.precis,
            &days.min_temp,
            &days.max_temp,
            &days.uv_max as &[Option<f64>],
            &days.rain_probability as &[Option<i32>],
            &days.rain_start_range as &[Option<i32>],
            &days.rain_end_range as &[Option<i32>],
            &days.rain_range_code as &[Option<String>],
            &days.wind_max_speed as &[Option<f64>],
            &days.first_light as &[Option<DateTime<Utc>>],
            &days.sunrise as &[Option<DateTime<Utc>>],
            &days.sunset as &[Option<DateTime<Utc>>],
            &days.last_light as &[Option<DateTime<Utc>>],
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();

        let inserted_hours = sqlx::query!(
            r#"
            INSERT INTO willyweather_forecast_hour_history (
                location, forecast_time, temperature, wind_speed, wind_direction, wind_direction_text
            )
            SELECT * FROM UNNEST(
                $1::TEXT[], $2::TIMESTAMPTZ[], $3::DOUBLE PRECISION[],
                $4::DOUBLE PRECISION[], $5::DOUBLE PRECISION[], $6::TEXT[]
            )
            "#,
            &hours.location,
            &hours.time,
            &hours.temperature as &[Option<f64>],
            &hours.wind_speed as &[Option<f64>],
            &hours.wind_direction as &[Option<f64>],
            &hours.wind_direction_text as &[Option<String>],
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();

        Ok(inserted_days + inserted_hours)
    }

    #[tracing::instrument(skip_all, name = "db.willyweather.forecast", fields(location), err)]
    pub async fn forecast(
        &self,
        location: &str,
    ) -> Result<Option<Forecast>, WillyWeatherRepoError> {
        let today = Utc::now().with_timezone(&FORECAST_OFFSET).date_naive();

        let day_rows = sqlx::query!(
            r#"
            SELECT date, precis_code, precis, min_temp, max_temp, uv_max,
                   rain_probability, rain_start_range, rain_end_range, rain_range_code,
                   wind_max_speed, first_light, sunrise, sunset, last_light
            FROM willyweather_forecast_day
            WHERE location = $1 AND date >= $2
            ORDER BY date ASC
            "#,
            location,
            today,
        )
        .fetch_all(&self.db)
        .await?;

        if day_rows.is_empty() {
            return Ok(None);
        }

        let midnight = FORECAST_OFFSET
            .from_local_datetime(&today.and_time(NaiveTime::MIN))
            .single()
            .map_or_else(Utc::now, |midnight| midnight.with_timezone(&Utc));

        let hour_rows = sqlx::query!(
            r#"
            SELECT "time", temperature, wind_speed, wind_direction, wind_direction_text
            FROM willyweather_forecast_hour
            WHERE location = $1 AND "time" >= $2
            ORDER BY "time" ASC
            "#,
            location,
            midnight,
        )
        .fetch_all(&self.db)
        .await?;

        let days = day_rows
            .into_iter()
            .map(|row| ForecastDetails {
                date_time: local_midnight(row.date),
                emoji: precis_emoji(&row.precis_code),
                code: row.precis_code,
                description: row.precis,
                min: row.min_temp as i64,
                max: row.max_temp as i64,
                uv: row.uv_max,
                rain_probability: row.rain_probability.map(i64::from),
                rain_start_range: row.rain_start_range.map(i64::from),
                rain_end_range: row.rain_end_range.map(i64::from),
                rain_range_code: row.rain_range_code,
                wind_max_speed: row.wind_max_speed,
                first_light: row.first_light.map(local_string),
                sunrise: row.sunrise.map(local_string),
                sunset: row.sunset.map(local_string),
                last_light: row.last_light.map(local_string),
            })
            .collect();

        let hours = hour_rows
            .into_iter()
            .map(|row| ForecastHour {
                date_time: local_string(row.time),
                temperature: row.temperature,
                wind_speed: row.wind_speed,
                wind_direction: row.wind_direction,
                wind_direction_text: row.wind_direction_text,
            })
            .collect();

        Ok(Some(Forecast { days, hours }))
    }
}
