use crate::integrations::solar::types::{
    GenerationHistory, SolarCurrentResponse, SolarCurrentStatistics, SolarCurrentStatisticsAverages,
};
use crate::repo::SolarRepo;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta, TimeZone, Utc};
use chrono_tz::Australia::Perth;
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

const LATEST_WINDOW: TimeDelta = TimeDelta::days(1);

struct LocalDay {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

fn yesterday(now: DateTime<Utc>) -> LocalDay {
    let today = now.with_timezone(&Perth).date_naive();
    let midnight = |date: NaiveDate| {
        Perth
            .from_local_datetime(&date.and_time(NaiveTime::MIN))
            .earliest()
            .map_or(now, |local| local.with_timezone(&Utc))
    };

    LocalDay {
        start: midnight(today - TimeDelta::days(1)),
        end: midnight(today),
    }
}

async fn statistics_with(repo: &SolarRepo) -> Result<SolarCurrentStatistics, SolarQueryError> {
    let averages = repo.averages().await?;

    Ok(SolarCurrentStatistics {
        averages: SolarCurrentStatisticsAverages {
            last_15_mins: round(averages.last_15_mins),
            last_1_hour: round(averages.last_1_hour),
            last_3_hours: round(averages.last_3_hours),
        },
    })
}

pub async fn statistics(db: &Pool<Postgres>) -> Result<SolarCurrentStatistics, SolarQueryError> {
    statistics_with(&SolarRepo::new(db.clone())).await
}

pub async fn current_wh(db: &Pool<Postgres>) -> Result<Option<f64>, SolarQueryError> {
    let latest = SolarRepo::new(db.clone())
        .latest_kpis(Utc::now() - LATEST_WINDOW)
        .await?;

    Ok(latest.map(|latest| latest.current_kwh))
}

pub async fn current(db: &Pool<Postgres>) -> Result<SolarCurrentResponse, SolarQueryError> {
    let repo = SolarRepo::new(db.clone());
    let now = Utc::now();
    let day = yesterday(now);

    let (latest, yesterday_kwh, statistics) = futures::try_join!(
        async { Ok::<_, SolarQueryError>(repo.latest_kpis(now - LATEST_WINDOW).await?) },
        async { Ok::<_, SolarQueryError>(repo.last_today_kwh_between(day.start, day.end).await?) },
        statistics_with(&repo),
    )?;

    let latest = latest.ok_or(SolarQueryError::NoData)?;

    Ok(SolarCurrentResponse {
        yesterday_production_kwh: yesterday_kwh.unwrap_or(0f64),
        month_production_kwh: latest.month_kwh,
        current_production_wh: latest.current_kwh,
        today_production_kwh: latest.today_kwh,
        all_time_production_kwh: latest.total_kwh,
        uv_level: latest.uv_level,
        temperature: latest.temperature,
        statistics,
    })
}

pub const MAX_HISTORY_WINDOW: TimeDelta = TimeDelta::days(30);

fn clamp_since(since: DateTime<Utc>, now: DateTime<Utc>) -> DateTime<Utc> {
    since.max(now - MAX_HISTORY_WINDOW)
}

pub async fn history_since(
    db: &Pool<Postgres>,
    since: DateTime<Utc>,
) -> Result<Vec<GenerationHistory>, SolarQueryError> {
    let since = clamp_since(since, Utc::now());

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
    fn yesterday_spans_the_previous_perth_day() {
        let now = Utc.with_ymd_and_hms(2026, 8, 16, 2, 0, 0).unwrap();
        let day = yesterday(now);

        assert_eq!(
            day.start,
            Utc.with_ymd_and_hms(2026, 8, 14, 16, 0, 0).unwrap()
        );
        assert_eq!(
            day.end,
            Utc.with_ymd_and_hms(2026, 8, 15, 16, 0, 0).unwrap()
        );
    }

    #[test]
    fn a_recent_since_is_left_alone() {
        let now = Utc.with_ymd_and_hms(2026, 8, 16, 0, 0, 0).unwrap();
        let recent = now - TimeDelta::days(1);

        assert_eq!(clamp_since(recent, now), recent);
    }
}
