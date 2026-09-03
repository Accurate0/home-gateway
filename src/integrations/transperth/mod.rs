use std::collections::HashSet;
use std::sync::Arc;

use arc_swap::ArcSwap;
use chrono::{DateTime, NaiveDate, TimeDelta, Timelike, Utc};
use chrono_tz::Australia::Perth;
use moka::future::Cache;
use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;

use crate::http::get_traced_http_client;
use crate::settings::{TransperthRoute, TransperthSettings};

pub mod auth;
pub mod departures;
pub mod gtfs;
pub mod realtime;

#[cfg(test)]
mod tests;

pub const TIMETABLE_KEY: &str = "transperth/timetable.json";
pub const GTFS_URL: &str =
    "http://www.transperth.wa.gov.au/TimetablePDFs/GoogleTransit/Production/google_transit.zip";
pub const USER_AGENT: &str = "okhttp/4.9.2";

const EARLY_HOURS: u32 = 5;
const ROUTE_CAPACITY: u64 = 32;
const ROUTE_TTL_REFRESHES: i32 = 4;
const PTA_TIMETABLE_CAPACITY: u64 = 16;
const PTA_TIMETABLE_TTL: std::time::Duration = std::time::Duration::from_secs(6 * 3600);

#[derive(thiserror::Error, Debug)]
pub enum TransperthError {
    #[error(transparent)]
    Http(#[from] crate::http::HttpCreationError),
    #[error(transparent)]
    Request(#[from] reqwest_middleware::Error),
    #[error("transperth returned {status}: {body}")]
    Status {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("gtfs archive is missing {0}")]
    MissingGtfsEntry(String),
    #[error("gtfs archive is missing the {0} column")]
    MissingGtfsColumn(&'static str),
    #[error("no gtfs stops matched any configured transperth station name")]
    NoWatchedStops,
    #[error("no timetable index has been loaded")]
    MissingTimetable,
    #[error("unknown transperth routes id: {0}")]
    UnknownRoute(String),
    #[error("transperth refresh intervals must be positive")]
    InvalidRefresh,
}

pub struct GtfsArchive {
    pub bytes: Vec<u8>,
    pub etag: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Departure {
    pub trip_id: String,
    pub line: String,
    pub headsign: String,
    pub platform: Option<String>,
    pub scheduled_departure: DateTime<Utc>,
    pub live_departure: Option<DateTime<Utc>>,
}

impl Departure {
    pub fn effective_departure(&self) -> DateTime<Utc> {
        self.live_departure.unwrap_or(self.scheduled_departure)
    }

    pub fn delay_minutes(&self) -> Option<i64> {
        self.live_departure
            .map(|live| (live - self.scheduled_departure).num_minutes())
    }

    pub fn minutes_away(&self, now: DateTime<Utc>) -> i64 {
        (self.effective_departure() - now).num_minutes()
    }
}

#[derive(Debug, Clone)]
pub struct RouteDepartures {
    pub id: String,
    pub origin: String,
    pub destination: String,
    pub updated_at: DateTime<Utc>,
    pub departures: Vec<Departure>,
}

#[derive(Clone)]
pub struct Transperth {
    client: ClientWithMiddleware,
    realtime_api_key: Option<String>,
    reference_data_api_key: Option<String>,
    routes: Arc<Vec<TransperthRoute>>,
    horizon: TimeDelta,
    index: Arc<ArcSwap<Option<gtfs::TimetableIndex>>>,
    route_departures: Cache<String, Arc<RouteDepartures>>,
    pta_timetables: Cache<(String, NaiveDate), Arc<realtime::PtaTimetable>>,
}

impl Transperth {
    pub fn new(settings: &TransperthSettings) -> Result<Self, TransperthError> {
        let realtime_api_key = settings
            .realtime_api_key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .map(str::to_owned);

        if realtime_api_key.is_none() {
            tracing::warn!("transperth realtime key absent, live departures disabled");
        }

        tracing::info!(
            "transperth integration enabled ({} routes)",
            settings.routes.len()
        );

        let route_ttl = settings
            .refresh_off_peak
            .checked_mul(ROUTE_TTL_REFRESHES)
            .and_then(|ttl| ttl.to_std().ok())
            .ok_or(TransperthError::InvalidRefresh)?;

        Ok(Self {
            client: get_traced_http_client()?,
            realtime_api_key,
            reference_data_api_key: settings
                .reference_data_api_key
                .as_deref()
                .map(str::trim)
                .filter(|key| !key.is_empty())
                .map(str::to_owned),
            routes: Arc::new(settings.routes.clone()),
            horizon: settings.horizon,
            index: Arc::new(ArcSwap::from_pointee(None)),
            route_departures: Cache::builder()
                .max_capacity(ROUTE_CAPACITY)
                .time_to_live(route_ttl)
                .build(),
            pta_timetables: Cache::builder()
                .max_capacity(PTA_TIMETABLE_CAPACITY)
                .time_to_live(PTA_TIMETABLE_TTL)
                .build(),
        })
    }

    pub fn watched_station_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .routes
            .iter()
            .flat_map(|routes| [routes.from.clone(), routes.to.clone()])
            .collect();

        names.sort();
        names.dedup();

        names
    }

    pub fn has_index(&self) -> bool {
        self.index.load().is_some()
    }

    pub fn store_index(&self, index: gtfs::TimetableIndex) {
        self.index.store(Arc::new(Some(index)));
    }

    pub async fn route(&self, id: &str) -> Option<Arc<RouteDepartures>> {
        self.route_departures.get(id).await
    }

    pub async fn route_departures(&self) -> Vec<Arc<RouteDepartures>> {
        let mut departures = Vec::new();

        for route in self.routes.iter() {
            if let Some(route_departures) = self.route_departures.get(&route.id).await {
                departures.push(route_departures);
            }
        }

        departures
    }

    pub fn routes(&self) -> &[TransperthRoute] {
        &self.routes
    }

    #[instrument(skip(self))]
    pub async fn download_gtfs(
        &self,
        etag: Option<&str>,
    ) -> Result<Option<GtfsArchive>, TransperthError> {
        let mut request = self.client.get(GTFS_URL).header("user-agent", USER_AGENT);

        if let Some(etag) = etag {
            request = request.header("if-none-match", etag);
        }

        let response = request.send().await?;

        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            tracing::info!("transperth gtfs unchanged");
            return Ok(None);
        }

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TransperthError::Status { status, body });
        }

        let etag = response
            .headers()
            .get("etag")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);

        let bytes = response
            .bytes()
            .await
            .map_err(reqwest_middleware::Error::from)?;

        Ok(Some(GtfsArchive {
            bytes: bytes.to_vec(),
            etag,
        }))
    }

    pub async fn refresh_routes(&self, now: DateTime<Utc>) -> Result<(), TransperthError> {
        let index = self.index.load();

        let Some(index) = index.as_ref() else {
            return Err(TransperthError::MissingTimetable);
        };

        let dates = service_dates(now);

        for routes in self.routes.iter() {
            let mut departures = departures::build(index, routes, now, self.horizon, &dates);

            if let Err(error) = self.apply_realtime(routes, &mut departures, &dates).await {
                tracing::warn!(
                    "transperth realtime overlay failed for {}: {error}",
                    routes.id
                );
            }

            self.route_departures
                .insert(
                    routes.id.clone(),
                    Arc::new(RouteDepartures {
                        id: routes.id.clone(),
                        origin: routes.from.clone(),
                        destination: routes.to.clone(),
                        updated_at: now,
                        departures,
                    }),
                )
                .await;
        }

        Ok(())
    }

    async fn apply_realtime(
        &self,
        routes: &TransperthRoute,
        departures: &mut [Departure],
        dates: &[NaiveDate],
    ) -> Result<(), TransperthError> {
        let (Some(realtime_key), Some(reference_key), Some(route_group)) = (
            self.realtime_api_key.as_deref(),
            self.reference_data_api_key.as_deref(),
            routes.route_group.as_deref(),
        ) else {
            return Ok(());
        };

        if departures.is_empty() {
            return Ok(());
        }

        for date in dates {
            let timetable = self
                .pta_timetable(reference_key, route_group, *date)
                .await?;

            realtime::overlay(&self.client, realtime_key, &timetable, *date, departures).await?;
        }

        Ok(())
    }

    async fn pta_timetable(
        &self,
        reference_key: &str,
        route_group: &str,
        date: NaiveDate,
    ) -> Result<Arc<realtime::PtaTimetable>, TransperthError> {
        let key = (route_group.to_owned(), date);

        if let Some(timetable) = self.pta_timetables.get(&key).await {
            return Ok(timetable);
        }

        let timetable = Arc::new(
            realtime::fetch_timetable(&self.client, reference_key, route_group, date).await?,
        );

        self.pta_timetables.insert(key, timetable.clone()).await;

        Ok(timetable)
    }
}

fn service_dates(now: DateTime<Utc>) -> Vec<NaiveDate> {
    let local = now.with_timezone(&Perth);
    let today = local.date_naive();

    if local.hour() < EARLY_HOURS {
        vec![today - TimeDelta::days(1), today]
    } else {
        vec![today]
    }
}

pub fn normalise_station(value: &str) -> String {
    let cleaned: String = value
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect();

    cleaned
        .split_whitespace()
        .filter(|word| !matches!(*word, "stn" | "station"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn station_stop_ids(index: &gtfs::TimetableIndex, name: &str) -> HashSet<String> {
    index
        .stations
        .get(&normalise_station(name))
        .map(|ids| ids.iter().cloned().collect())
        .unwrap_or_default()
}
