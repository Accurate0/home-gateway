use crate::{
    event_bus::{EventBusMessage, FuelChange},
    integrations::fuelwatch::{FuelWatch, types::FuelSite},
    settings::FuelWatchSettings,
    state::AppState,
};
use ractor::Actor;
use std::collections::HashMap;
use uuid::Uuid;

pub enum FuelWatchMessage {
    Poll,
}

pub struct FuelWatchActorState {
    postcode: i32,
    prices: HashMap<i32, (f64, Option<f64>)>,
    cheapest: Option<(i32, f64)>,
}

impl FuelWatchActorState {
    pub fn new(postcode: i32) -> Self {
        Self {
            postcode,
            prices: HashMap::new(),
            cheapest: None,
        }
    }

    pub fn apply(&mut self, sites: &[FuelSite]) -> Vec<EventBusMessage> {
        let mut events = Vec::new();

        for site in sites.iter().filter(|site| site.postcode == self.postcode) {
            if let Some((price, tomorrow)) = self.prices.get(&site.site_id).copied() {
                if site.price < price {
                    events.push(event(FuelChange::Drop, site, price, site.price));
                }

                if let Some(next) = site.price_tomorrow
                    && Some(next) != tomorrow
                {
                    let change = if next < site.price {
                        Some(FuelChange::TomorrowLower)
                    } else if next > site.price {
                        Some(FuelChange::TomorrowHigher)
                    } else {
                        None
                    };

                    if let Some(change) = change {
                        events.push(event(change, site, site.price, next));
                    }
                }
            }

            self.prices
                .insert(site.site_id, (site.price, site.price_tomorrow));
        }

        let cheapest = sites
            .iter()
            .filter(|site| site.postcode == self.postcode)
            .min_by(|a, b| a.price.total_cmp(&b.price).then(a.site_id.cmp(&b.site_id)));

        if let Some(site) = cheapest {
            if let Some((previous_site, previous_price)) = self.cheapest
                && (previous_site != site.site_id || previous_price != site.price)
            {
                events.push(event(
                    FuelChange::Cheapest,
                    site,
                    previous_price,
                    site.price,
                ));
            }

            self.cheapest = Some((site.site_id, site.price));
        }

        events
    }
}

fn event(change: FuelChange, site: &FuelSite, old_price: f64, new_price: f64) -> EventBusMessage {
    EventBusMessage::FuelWatch {
        event_id: Uuid::new_v4(),
        change,
        site_id: site.site_id,
        name: site.name.clone(),
        brand: site.brand.clone(),
        suburb: site.suburb.clone(),
        address: site.address.clone(),
        old_price,
        new_price,
    }
}

pub struct FuelWatchActor {
    pub shared_actor_state: AppState,
    pub fuelwatch: FuelWatch,
}

impl FuelWatchActor {
    pub const NAME: &str = "fuelwatch";

    async fn poll(
        &self,
        state: &mut FuelWatchActorState,
    ) -> Result<(), ractor::ActorProcessingErr> {
        let sites = self.fuelwatch.fetch_sites().await?;

        if sites.is_empty() {
            tracing::warn!("fuelwatch returned no priced sites, leaving the stored set alone");
            return Ok(());
        }

        let repo = self.shared_actor_state.repos.fuelwatch();
        let mut tx = self.shared_actor_state.db.begin().await?;

        let stored = repo.replace_sites(&mut tx, &sites).await?;
        let appended = repo.append_history(&mut tx, &sites).await?;

        tx.commit().await?;

        tracing::info!("stored {stored} fuelwatch sites and appended {appended} history rows");

        for event in state.apply(&sites) {
            self.shared_actor_state.event_bus.publish(event);
        }

        Ok(())
    }
}

impl Actor for FuelWatchActor {
    type Msg = FuelWatchMessage;
    type State = FuelWatchActorState;
    type Arguments = FuelWatchSettings;

    async fn pre_start(
        &self,
        myself: ractor::ActorRef<Self::Msg>,
        settings: Self::Arguments,
    ) -> Result<Self::State, ractor::ActorProcessingErr> {
        let stored = self
            .shared_actor_state
            .repos
            .fuelwatch()
            .sites_for_postcode(settings.postcode, None)
            .await?;

        let mut state = FuelWatchActorState::new(settings.postcode);
        state.apply(&stored);

        myself.send_interval(settings.refresh.to_std()?, || FuelWatchMessage::Poll);

        Ok(state)
    }

    #[tracing::instrument(parent = None, name = "fuelwatch-actor", skip(self, _myself, message, state))]
    async fn handle(
        &self,
        _myself: ractor::ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ractor::ActorProcessingErr> {
        match message {
            FuelWatchMessage::Poll => {
                let started = std::time::Instant::now();

                match self.poll(state).await {
                    Ok(()) => {
                        crate::metrics::record_integration_poll(
                            "fuelwatch",
                            "success",
                            started.elapsed(),
                        );
                    }
                    Err(e) => {
                        tracing::error!("error polling fuelwatch: {e}");
                        crate::metrics::record_integration_poll(
                            "fuelwatch",
                            "error",
                            started.elapsed(),
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POSTCODE: i32 = 6164;

    fn site(site_id: i32, postcode: i32, price: f64, price_tomorrow: Option<f64>) -> FuelSite {
        FuelSite {
            site_id,
            name: format!("site {site_id}"),
            brand: "Brand".to_owned(),
            suburb: "Suburb".to_owned(),
            postcode,
            address: "1 Road".to_owned(),
            price,
            price_tomorrow,
            latitude: 0.0,
            longitude: 0.0,
        }
    }

    fn changes(events: &[EventBusMessage]) -> Vec<(FuelChange, i32, f64, f64)> {
        events
            .iter()
            .map(|event| match event {
                EventBusMessage::FuelWatch {
                    change,
                    site_id,
                    old_price,
                    new_price,
                    ..
                } => (*change, *site_id, *old_price, *new_price),
                other => panic!("expected a fuelwatch event, got {}", other.kind()),
            })
            .collect()
    }

    #[test]
    fn the_first_sighting_is_silent() {
        let mut state = FuelWatchActorState::new(POSTCODE);

        let events = state.apply(&[site(1, POSTCODE, 190.0, Some(180.0))]);

        assert!(events.is_empty());
    }

    #[test]
    fn a_price_drop_and_a_new_cheapest_site_are_published() {
        let mut state = FuelWatchActorState::new(POSTCODE);
        state.apply(&[
            site(1, POSTCODE, 190.0, None),
            site(2, POSTCODE, 195.0, None),
        ]);

        let events = state.apply(&[
            site(1, POSTCODE, 190.0, None),
            site(2, POSTCODE, 185.0, None),
        ]);

        assert_eq!(
            changes(&events),
            [
                (FuelChange::Drop, 2, 195.0, 185.0),
                (FuelChange::Cheapest, 2, 190.0, 185.0),
            ]
        );
    }

    #[test]
    fn tomorrow_prices_publish_once_when_they_appear_or_change() {
        let mut state = FuelWatchActorState::new(POSTCODE);
        state.apply(&[site(1, POSTCODE, 190.0, None)]);

        let appeared = state.apply(&[site(1, POSTCODE, 190.0, Some(175.0))]);
        let repeated = state.apply(&[site(1, POSTCODE, 190.0, Some(175.0))]);
        let raised = state.apply(&[site(1, POSTCODE, 190.0, Some(199.0))]);

        assert_eq!(
            changes(&appeared),
            [(FuelChange::TomorrowLower, 1, 190.0, 175.0)]
        );
        assert!(repeated.is_empty());
        assert_eq!(
            changes(&raised),
            [(FuelChange::TomorrowHigher, 1, 190.0, 199.0)]
        );
    }

    #[test]
    fn sites_outside_the_postcode_are_ignored() {
        let mut state = FuelWatchActorState::new(POSTCODE);
        state.apply(&[site(1, 6000, 190.0, None)]);

        let events = state.apply(&[site(1, 6000, 150.0, Some(140.0))]);

        assert!(events.is_empty());
    }
}
