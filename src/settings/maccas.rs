use serde::Deserialize;

use super::notify::{NotifyRef, NotifySource, NotifyTargets, resolve_notify};
use super::yes;

#[derive(Debug, Clone)]
pub struct MaccasOfferSettings {
    pub match_names: Vec<String>,
    pub notify: Vec<NotifySource>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct RawMaccasOfferSettings {
    match_names: Vec<String>,
    #[serde(default)]
    notify: Vec<NotifyRef>,
    #[serde(default = "yes")]
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct MaccasSettings {
    pub offers: Vec<MaccasOfferSettings>,
    pub webhook_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct RawMaccasSettings {
    offers: Vec<RawMaccasOfferSettings>,
    webhook_secret: String,
}

impl RawMaccasSettings {
    pub(super) fn resolve(self, targets: &NotifyTargets) -> Result<MaccasSettings, String> {
        let offers = self
            .offers
            .into_iter()
            .map(|o| {
                Ok(MaccasOfferSettings {
                    match_names: o.match_names,
                    enabled: o.enabled,
                    notify: resolve_notify(o.notify, targets)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(MaccasSettings {
            offers,
            webhook_secret: self.webhook_secret,
        })
    }
}
