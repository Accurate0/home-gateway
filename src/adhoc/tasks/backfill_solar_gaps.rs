use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, TimeDelta, Utc};
use chrono_tz::Australia::Perth;

use crate::adhoc::{AdhocTask, AdhocTaskContext, AdhocTaskError};
use crate::adhoc_task_source;
use crate::integrations::solar::goodwe::{GoodWeSemsAPI, GoodWeSemsAPIError};

const SAMPLE_HOURS: f64 = 5.0 / 60.0;
const MAX_SAMPLE_AGE: TimeDelta = TimeDelta::minutes(10);
const MIN_GAP: TimeDelta = TimeDelta::minutes(10);
const STEP: TimeDelta = TimeDelta::minutes(1);

pub struct BackfillSolarGaps;

#[async_trait::async_trait]
impl AdhocTask for BackfillSolarGaps {
    fn ordinal(&self) -> i64 {
        5
    }

    fn name(&self) -> &'static str {
        "backfill_solar_gaps"
    }

    fn source(&self) -> &'static str {
        adhoc_task_source!()
    }

    async fn run(&self, ctx: &mut AdhocTaskContext<'_>) -> Result<(), AdhocTaskError> {
        let Some(goodwe) = ctx.goodwe else {
            tracing::info!("solar integration is disabled, nothing to backfill");
            return Ok(());
        };

        let mut gaps = sqlx::query!(
            "SELECT prev AS \"start!\", time AS \"end!\" FROM ( \
                 SELECT time, lag(time) OVER (ORDER BY time) AS prev FROM solar_data_tsdb \
                 WHERE time > now() - interval '30 days' \
             ) t WHERE time - prev > $1",
            sqlx::postgres::types::PgInterval::try_from(MIN_GAP)
                .map_err(|e| AdhocTaskError::Failed(e.to_string()))?
        )
        .fetch_all(&mut **ctx.tx)
        .await?
        .into_iter()
        .map(|row| (row.start, row.end))
        .collect::<Vec<_>>();

        let now = Utc::now();
        if let Some(last) = sqlx::query_scalar!("SELECT max(time) FROM solar_data_tsdb")
            .fetch_one(&mut **ctx.tx)
            .await?
            && now - last > MIN_GAP
        {
            gaps.push((last, now));
        }

        let times = gaps
            .iter()
            .flat_map(|(start, end)| {
                std::iter::successors(Some(*start + STEP), |time| Some(*time + STEP))
                    .take_while(move |time| time < end)
            })
            .collect::<Vec<_>>();

        let (Some(first), Some(last)) = (times.first(), times.last()) else {
            return Ok(());
        };

        tracing::info!(
            "filling {} solar readings across {} gaps from {first} to {last} from sems+",
            times.len(),
            gaps.len()
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
        let raw_data = serde_json::json!({ "backfill": "semsplus_gap" });

        for batch in readings.chunks(batch_size) {
            sqlx::query!("SET LOCAL timescaledb.max_tuples_decompressed_per_dml_transaction = 0")
                .execute(&mut **ctx.tx)
                .await?;

            sqlx::query!(
                "INSERT INTO solar_data_tsdb \
                     (time, current_kwh, today_kwh, month_kwh, total_kwh, raw_data) \
                 SELECT v.time, v.current_kwh, v.today_kwh, v.month_kwh, v.total_kwh, $6 \
                 FROM unnest($1::timestamptz[], $2::float8[], $3::float8[], $4::float8[], $5::float8[]) \
                     AS v(time, current_kwh, today_kwh, month_kwh, total_kwh)",
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

            tracing::info!("filled {} solar readings", batch.len());
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
