use std::sync::Arc;

use async_graphql::{Object, SimpleObject};
use chrono::{DateTime, Utc};

use crate::integrations::transperth::{RouteDepartures, Transperth};

pub struct TransperthObject;

#[Object]
impl TransperthObject {
    async fn routes(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<RouteDeparturesObject>> {
        let Some(transperth) = ctx.data::<Option<Transperth>>()? else {
            return Ok(Vec::new());
        };

        Ok(transperth
            .route_departures()
            .await
            .into_iter()
            .map(RouteDeparturesObject::from_route)
            .collect())
    }

    async fn route(
        &self,
        ctx: &async_graphql::Context<'_>,
        id: String,
    ) -> async_graphql::Result<Option<RouteDeparturesObject>> {
        let Some(transperth) = ctx.data::<Option<Transperth>>()? else {
            return Ok(None);
        };

        Ok(transperth
            .route(&id)
            .await
            .map(RouteDeparturesObject::from_route))
    }
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct RouteDeparturesObject {
    pub id: String,
    pub origin: String,
    pub destination: String,
    pub updated_at: DateTime<Utc>,
    pub stale: bool,
    pub departures: Vec<DepartureObject>,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct DepartureObject {
    pub line: String,
    pub headsign: String,
    pub platform: Option<String>,
    pub scheduled_departure: DateTime<Utc>,
    pub live_departure: Option<DateTime<Utc>>,
    pub delay_minutes: Option<i64>,
    pub minutes_away: i64,
    pub live: bool,
}

impl RouteDeparturesObject {
    fn from_route(route: Arc<RouteDepartures>) -> Self {
        let now = Utc::now();

        let departures = route
            .departures
            .iter()
            .map(|departure| DepartureObject {
                line: departure.line.clone(),
                headsign: departure.headsign.clone(),
                platform: departure.platform.clone(),
                scheduled_departure: departure.scheduled_departure,
                live_departure: departure.live_departure,
                delay_minutes: departure.delay_minutes(),
                minutes_away: departure.minutes_away(now),
                live: departure.live_departure.is_some(),
            })
            .collect();

        Self {
            id: route.id.clone(),
            origin: route.origin.clone(),
            destination: route.destination.clone(),
            updated_at: route.updated_at,
            stale: route.departures.is_empty(),
            departures,
        }
    }
}
