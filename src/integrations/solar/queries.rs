use crate::integrations::solar::goodwe::types::PlantDetailsByPowerStationIdResponse;
use crate::integrations::solar::types::{
    GenerationHistory, SolarCurrentResponse, SolarCurrentStatistics, SolarCurrentStatisticsAverages,
};
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use tracing::Instrument;

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
    let row = sqlx::query!(
        "SELECT avg(current_kwh) AS avg FROM solar_data_tsdb \
         WHERE time > now() - MAKE_INTERVAL(mins => $1)",
        minutes
    )
    .fetch_optional(db)
    .instrument(tracing::info_span!("solar_average", time_in_mins = minutes))
    .await?;

    Ok(row.and_then(|r| r.avg))
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
    let latest = sqlx::query!(
        "SELECT raw_data, temperature, uv_level FROM solar_data_tsdb ORDER BY time DESC LIMIT 1"
    )
    .fetch_optional(db)
    .instrument(tracing::info_span!("get_latest_solar_data"))
    .await?
    .ok_or(SolarQueryError::NoData)?;

    let raw_data = serde_json::from_value::<PlantDetailsByPowerStationIdResponse>(latest.raw_data)?;

    let yesterday = sqlx::query!(
        "SELECT raw_data FROM solar_data_tsdb \
         WHERE (time AT TIME ZONE 'Australia/Perth')::date \
             = (now() AT TIME ZONE 'Australia/Perth')::date - INTEGER '1' \
         ORDER BY time DESC LIMIT 1"
    )
    .fetch_optional(db)
    .instrument(tracing::info_span!("get_yesterday_results"))
    .await?;

    let yesterday_production_kwh = match yesterday {
        Some(row) => {
            serde_json::from_value::<PlantDetailsByPowerStationIdResponse>(row.raw_data)?
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

pub async fn history_since(
    db: &Pool<Postgres>,
    since: DateTime<Utc>,
) -> Result<Vec<GenerationHistory>, SolarQueryError> {
    let history = sqlx::query!(
        "SELECT avg(current_kwh) AS avg_wh, avg(uv_level) AS avg_uv_level, \
                avg(temperature) AS avg_temp, time_bucket('5 minutes', time) AS bucket_time \
         FROM solar_data_tsdb WHERE time >= $1 \
         GROUP BY bucket_time ORDER BY bucket_time ASC",
        since
    )
    .fetch_all(db)
    .instrument(tracing::info_span!("solar_history_since"))
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

    let (today, yesterday) = sqlx::query!(
        "SELECT avg(current_kwh) AS avg_wh, avg(uv_level) AS avg_uv_level, \
                avg(temperature) AS avg_temp, time_bucket('5 minutes', time) AS bucket_time \
         FROM solar_data_tsdb \
         WHERE (time AT TIME ZONE 'Australia/Perth')::date \
             > ((now() AT TIME ZONE 'Australia/Perth')::date - 2) \
         GROUP BY bucket_time ORDER BY bucket_time ASC"
    )
    .fetch_all(db)
    .instrument(tracing::info_span!("solar_history_two_days"))
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
    use serde_json::json;

    async fn insert(db: &Pool<Postgres>, at: DateTime<Utc>, kwh: f64, uv: Option<f64>) {
        sqlx::query!(
            "INSERT INTO solar_data_tsdb (current_kwh, raw_data, uv_level, temperature, time) \
             VALUES ($1, $2, $3, $4, $5)",
            kwh,
            json!({"data": {"kpi": {
                "month_generation": 1.0, "pac": kwh, "power": 2.0, "total_power": 3.0,
                "day_income": 0.0, "total_income": 0.0, "yield_rate": 0.0, "currency": "AUD"
            }}}),
            uv,
            Some(20.0),
            at
        )
        .execute(db)
        .await
        .unwrap();
    }

    #[sqlx::test]
    async fn history_buckets_align_to_five_minute_utc_boundaries(db: Pool<Postgres>) {
        let base = Utc.with_ymd_and_hms(2026, 3, 1, 4, 2, 30).unwrap();
        insert(&db, base, 100.0, Some(5.0)).await;
        insert(&db, base + chrono::Duration::minutes(1), 200.0, Some(7.0)).await;

        let history = history_since(&db, base - chrono::Duration::hours(1))
            .await
            .unwrap();

        assert_eq!(history.len(), 1);
        let point = &history[0];

        assert_eq!(
            point.at,
            Utc.with_ymd_and_hms(2026, 3, 1, 4, 0, 0)
                .unwrap()
                .naive_utc()
        );
        assert_eq!(point.wh, 150.0);
        assert_eq!(point.uv_level, Some(6.0));
        assert_eq!(point.timestamp, point.at.and_utc().timestamp_millis());
    }

    #[sqlx::test]
    async fn two_day_history_splits_on_perth_local_date(db: Pool<Postgres>) {
        let now_perth = Utc::now().with_timezone(&chrono_tz::Australia::Perth);

        let today_local_morning = now_perth
            .date_naive()
            .and_hms_opt(2, 0, 0)
            .unwrap()
            .and_local_timezone(chrono_tz::Australia::Perth)
            .unwrap()
            .with_timezone(&Utc);

        let yesterday_local_morning = today_local_morning - chrono::Duration::days(1);

        insert(&db, today_local_morning, 100.0, None).await;
        insert(&db, yesterday_local_morning, 50.0, None).await;

        let (today, yesterday) = history_last_two_days(&db).await.unwrap();

        assert_eq!(today.len(), 1, "expected one bucket today");
        assert_eq!(today[0].wh, 100.0);
        assert_eq!(yesterday.len(), 1, "expected one bucket yesterday");
        assert_eq!(yesterday[0].wh, 50.0);
    }

    #[sqlx::test]
    async fn averages_ignore_rows_outside_the_window(db: Pool<Postgres>) {
        let now = Utc::now();
        insert(&db, now - chrono::Duration::minutes(5), 100.0, None).await;
        insert(&db, now - chrono::Duration::minutes(45), 300.0, None).await;

        let stats = statistics(&db).await.unwrap();

        assert_eq!(stats.averages.last_15_mins, Some(100.0));
        assert_eq!(stats.averages.last_1_hour, Some(200.0));
        assert_eq!(stats.averages.last_3_hours, Some(200.0));
    }

    #[sqlx::test]
    async fn current_reports_no_data_on_an_empty_table(db: Pool<Postgres>) {
        assert!(matches!(current(&db).await, Err(SolarQueryError::NoData)));
    }
}
