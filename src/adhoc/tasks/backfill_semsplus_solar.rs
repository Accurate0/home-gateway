use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, TimeDelta, Utc};
use chrono_tz::Australia::Perth;

use crate::adhoc::{AdhocTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use crate::integrations::solar::goodwe::{GoodWeSemsAPI, GoodWeSemsAPIError};

const SAMPLE_HOURS: f64 = 5.0 / 60.0;
const MAX_SAMPLE_AGE: TimeDelta = TimeDelta::minutes(10);

pub struct BackfillSemsplusSolar;

#[async_trait::async_trait]
impl AdhocTask for BackfillSemsplusSolar {
    fn ordinal(&self) -> i64 {
        4
    }

    fn name(&self) -> &'static str {
        "backfill_semsplus_solar"
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<(), AdhocTaskError> {
        let Some(goodwe) = ctx.goodwe else {
            tracing::info!("solar integration is disabled, nothing to backfill");
            return Ok(());
        };

        let Some(cutoff) = sqlx::query_scalar!(
            "SELECT max(time) FROM solar_data_tsdb WHERE raw_data ? 'data' AND current_kwh > 0"
        )
        .fetch_one(&mut **ctx.tx)
        .await?
        else {
            return Ok(());
        };

        let times = sqlx::query_scalar!(
            "SELECT time FROM solar_data_tsdb WHERE time > $1 AND raw_data ? 'data' ORDER BY time",
            cutoff
        )
        .fetch_all(&mut **ctx.tx)
        .await?;

        let (Some(first), Some(last)) = (times.first(), times.last()) else {
            return Ok(());
        };

        tracing::info!(
            "rebuilding {} solar readings from {first} to {last} from sems+",
            times.len()
        );

        let history = History::fetch(goodwe, local_date(*first), local_date(*last))
            .await
            .map_err(|e| AdhocTaskError::Failed(e.to_string()))?;

        let readings = times
            .iter()
            .map(|time| history.reading(*time))
            .collect::<Vec<_>>();

        let batch_size = usize::try_from(ctx.settings.adhoc.batch_size)
            .unwrap_or(1000)
            .max(1);
        let raw_data = serde_json::json!({ "backfill": "semsplus" });

        for batch in readings.chunks(batch_size) {
            sqlx::query!("SET LOCAL timescaledb.max_tuples_decompressed_per_dml_transaction = 0")
                .execute(&mut **ctx.tx)
                .await?;

            sqlx::query!(
                "UPDATE solar_data_tsdb AS s SET \
                     current_kwh = v.current_kwh, today_kwh = v.today_kwh, \
                     month_kwh = v.month_kwh, total_kwh = v.total_kwh, raw_data = $6 \
                 FROM unnest($1::timestamptz[], $2::float8[], $3::float8[], $4::float8[], $5::float8[]) \
                     AS v(time, current_kwh, today_kwh, month_kwh, total_kwh) \
                 WHERE s.time = v.time",
                &batch.iter().map(|r| r.time).collect::<Vec<_>>(),
                &batch.iter().map(|r| r.current_w).collect::<Vec<_>>(),
                &batch.iter().map(|r| r.today_kwh).collect::<Vec<_>>(),
                &batch.iter().map(|r| r.month_kwh).collect::<Vec<_>>(),
                &batch.iter().map(|r| r.total_kwh).collect::<Vec<_>>(),
                raw_data
            )
            .execute(&mut **ctx.tx)
            .await?;

            ctx.commit_batch().await?;

            tracing::info!("rebuilt {} solar readings", batch.len());
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
struct Reading {
    time: DateTime<Utc>,
    current_w: f64,
    today_kwh: f64,
    month_kwh: f64,
    total_kwh: f64,
}

struct History {
    curves: BTreeMap<NaiveDate, Vec<(NaiveDateTime, f64)>>,
    daily: BTreeMap<NaiveDate, f64>,
    today: NaiveDate,
    start_of_today_total: f64,
}

impl History {
    async fn fetch(
        goodwe: &GoodWeSemsAPI,
        first: NaiveDate,
        last: NaiveDate,
    ) -> Result<Self, GoodWeSemsAPIError> {
        let today = local_date(Utc::now());
        let now = goodwe.get_solar_data().await?;

        let mut daily = BTreeMap::new();
        let mut month = month_start(first);
        while month <= today {
            let next = month_start(month + TimeDelta::days(32));
            let end = (next - TimeDelta::days(1)).min(today);
            daily.extend(goodwe.daily_production(month, end).await?);
            month = next;
        }

        let mut curves = BTreeMap::new();
        for date in first.iter_days().take_while(|date| *date <= last) {
            curves.insert(date, goodwe.power_curve(date).await?);
        }

        Ok(Self {
            curves,
            daily,
            today,
            start_of_today_total: now.total_kwh - now.today_kwh,
        })
    }

    fn daily_sum(&self, from: NaiveDate, until: NaiveDate) -> f64 {
        self.daily.range(from..until).map(|(_, kwh)| kwh).sum()
    }

    fn reading(&self, time: DateTime<Utc>) -> Reading {
        let local = time.with_timezone(&Perth).naive_local();
        let date = local.date();
        let curve = self
            .curves
            .get(&date)
            .map(Vec::as_slice)
            .unwrap_or_default();

        let current_w = curve
            .iter()
            .rev()
            .find(|(at, _)| *at <= local)
            .filter(|(at, _)| local - *at <= MAX_SAMPLE_AGE)
            .map_or(0.0, |(_, kw)| kw * 1000.0);

        let produced = |points: &mut dyn Iterator<Item = &(NaiveDateTime, f64)>| {
            points.map(|(_, kw)| kw * SAMPLE_HOURS).sum::<f64>()
        };
        let so_far = produced(&mut curve.iter().filter(|(at, _)| *at <= local));
        let whole_day = produced(&mut curve.iter());
        let day_total = self.daily.get(&date).copied().unwrap_or(whole_day);
        let today_kwh = if whole_day > 0.0 {
            so_far * day_total / whole_day
        } else {
            0.0
        };

        Reading {
            time,
            current_w,
            today_kwh,
            month_kwh: self.daily_sum(month_start(date), date) + today_kwh,
            total_kwh: self.start_of_today_total - self.daily_sum(date, self.today) + today_kwh,
        }
    }
}

fn local_date(time: DateTime<Utc>) -> NaiveDate {
    time.with_timezone(&Perth).date_naive()
}

fn month_start(date: NaiveDate) -> NaiveDate {
    date.with_day(1).unwrap_or(date)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, day).unwrap()
    }

    fn local(day: u32, hour: u32, minute: u32) -> NaiveDateTime {
        date(day).and_hms_opt(hour, minute, 0).unwrap()
    }

    fn utc(day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        Perth
            .from_local_datetime(&local(day, hour, minute))
            .unwrap()
            .with_timezone(&Utc)
    }

    fn history() -> History {
        History {
            curves: BTreeMap::from([(
                date(3),
                vec![
                    (local(3, 12, 0), 6.0),
                    (local(3, 12, 5), 6.0),
                    (local(3, 12, 10), 12.0),
                ],
            )]),
            daily: BTreeMap::from([
                (date(1), 30.0),
                (date(2), 20.0),
                (date(3), 4.0),
                (date(4), 10.0),
            ]),
            today: date(4),
            start_of_today_total: 1000.0,
        }
    }

    #[test]
    fn rebuilds_a_reading_mid_curve() {
        let reading = history().reading(utc(3, 12, 7));

        assert_eq!(reading.current_w, 6000.0);
        assert_eq!(reading.today_kwh, 2.0);
        assert_eq!(reading.month_kwh, 52.0);
        assert_eq!(reading.total_kwh, 998.0);
    }

    #[test]
    fn night_readings_keep_the_day_total_and_no_power() {
        let reading = history().reading(utc(3, 21, 0));

        assert_eq!(reading.current_w, 0.0);
        assert_eq!(reading.today_kwh, 4.0);
        assert_eq!(reading.total_kwh, 1000.0);
    }

    #[test]
    fn days_without_a_curve_produce_nothing() {
        let reading = history().reading(utc(2, 12, 0));

        assert_eq!(reading.today_kwh, 0.0);
        assert_eq!(reading.month_kwh, 30.0);
        assert_eq!(reading.total_kwh, 976.0);
    }
}
