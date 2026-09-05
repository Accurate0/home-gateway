use crate::integrations::solar::goodwe::types::PlantDetailsByPowerStationIdResponse;
use crate::integrations::solar::types::{
    GenerationHistory, SolarCurrentResponse, SolarCurrentStatistics, SolarCurrentStatisticsAverages,
};
use crate::repo::SolarRepo;
use chrono::{DateTime, TimeDelta, Utc};
use sqlx::{Pool, Postgres};

#[derive(thiserror::Error, Debug)]
pub enum SolarQueryError {
    #[error("a database error occurred: {0}")]
    Database(#[from] sqlx::Error),
    #[error("could not decode stored data: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("no solar data recorded yet")]
    NoData,
}

fn round(n: Option<f64>) -> Option<f64> {
    n.map(|n| (n * 100.0).round() / 100.0)
}

pub async fn average_for_last_n_minutes(
    db: &Pool<Postgres>,
    minutes: i32,
) -> Result<Option<f64>, SolarQueryError> {
    let row = SolarRepo::new(db.clone())
        .average_for_last_n_minutes(minutes)
        .await?;

    Ok(row)
}

pub async fn statistics(db: &Pool<Postgres>) -> Result<SolarCurrentStatistics, SolarQueryError> {
    let (last_15_mins, last_1_hour, last_3_hours) = futures::try_join!(
        average_for_last_n_minutes(db, 15),
        average_for_last_n_minutes(db, 60),
        average_for_last_n_minutes(db, 180),
    )?;

    Ok(SolarCurrentStatistics {
        averages: SolarCurrentStatisticsAverages {
            last_15_mins: round(last_15_mins),
            last_1_hour: round(last_1_hour),
            last_3_hours: round(last_3_hours),
        },
    })
}

pub async fn current(db: &Pool<Postgres>) -> Result<SolarCurrentResponse, SolarQueryError> {
    let latest = SolarRepo::new(db.clone())
        .latest()
        .await?
        .ok_or(SolarQueryError::NoData)?;

    let raw_data = serde_json::from_value::<PlantDetailsByPowerStationIdResponse>(latest.raw_data)?;

    let yesterday = SolarRepo::new(db.clone()).yesterday_raw_data().await?;

    let yesterday_production_kwh = match yesterday {
        Some(raw_data) => {
            serde_json::from_value::<PlantDetailsByPowerStationIdResponse>(raw_data)?
                .data
                .kpi
                .power
        }
        None => 0f64,
    };

    Ok(SolarCurrentResponse {
        yesterday_production_kwh,
        month_production_kwh: raw_data.data.kpi.month_generation,
        current_production_wh: raw_data.data.kpi.pac,
        today_production_kwh: raw_data.data.kpi.power,
        all_time_production_kwh: raw_data.data.kpi.total_power,
        uv_level: latest.uv_level,
        temperature: latest.temperature,
        statistics: statistics(db).await?,
    })
}

pub const MAX_HISTORY_WINDOW: TimeDelta = TimeDelta::days(30);

pub fn clamp_since(since: DateTime<Utc>, now: DateTime<Utc>) -> DateTime<Utc> {
    since.max(now - MAX_HISTORY_WINDOW)
}

pub async fn history_since(
    db: &Pool<Postgres>,
    since: DateTime<Utc>,
) -> Result<Vec<GenerationHistory>, SolarQueryError> {
    let history = SolarRepo::new(db.clone())
        .buckets_since(since)
        .await?
        .into_iter()
        .filter_map(|r| {
            let bucket_time = r.bucket_time?;

            Some(GenerationHistory {
                uv_level: r.avg_uv_level,
                temperature: r.avg_temp,
                at: bucket_time.naive_utc(),
                wh: r.avg_wh?,
                timestamp: bucket_time.timestamp_millis(),
            })
        })
        .collect();

    Ok(history)
}

pub async fn history_last_two_days(
    db: &Pool<Postgres>,
) -> Result<(Vec<GenerationHistory>, Vec<GenerationHistory>), SolarQueryError> {
    let today_in_perth = Utc::now()
        .with_timezone(&chrono_tz::Australia::Perth)
        .date_naive();

    let (today, yesterday) = SolarRepo::new(db.clone())
        .buckets_last_two_days()
        .await?
        .into_iter()
        .filter_map(|r| {
            let bucket_time = r.bucket_time?;

            Some(GenerationHistory {
                uv_level: r.avg_uv_level,
                temperature: r.avg_temp,
                at: bucket_time.naive_utc(),
                wh: r.avg_wh?,
                timestamp: bucket_time.timestamp_millis(),
            })
        })
        .partition(|r| {
            r.at.and_utc()
                .with_timezone(&chrono_tz::Australia::Perth)
                .date_naive()
                == today_in_perth
        });

    Ok((today, yesterday))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn an_ancient_since_is_clamped_to_the_history_window() {
        let now = Utc.with_ymd_and_hms(2026, 8, 16, 0, 0, 0).unwrap();
        let epoch = Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap();

        assert_eq!(clamp_since(epoch, now), now - MAX_HISTORY_WINDOW);
    }

    #[test]
    fn a_recent_since_is_left_alone() {
        let now = Utc.with_ymd_and_hms(2026, 8, 16, 0, 0, 0).unwrap();
        let recent = now - TimeDelta::days(1);

        assert_eq!(clamp_since(recent, now), recent);
    }
}
