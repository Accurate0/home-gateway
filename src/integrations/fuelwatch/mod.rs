use std::sync::Arc;

use moka::future::Cache;
use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;

use crate::http::get_traced_http_client;
use crate::integrations::fuelwatch::types::{FuelSite, Site};
use crate::settings::FuelWatchSettings;

pub mod types;

const SITES_URL: &str = "https://www.fuelwatch.wa.gov.au/api/sites";
const PRODUCT_UNLEADED_91: &str = "1";
const CACHE_KEY: &str = "sites";

#[derive(thiserror::Error, Debug)]
pub enum FuelWatchError {
    #[error(transparent)]
    Http(#[from] crate::http::HttpCreationError),
    #[error(transparent)]
    Request(#[from] reqwest_middleware::Error),
    #[error("fuelwatch returned {status}: {body}")]
    Status {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("fuelwatch.cache_ttl must be positive")]
    InvalidCacheTtl,
    #[error(transparent)]
    Shared(Arc<FuelWatchError>),
}

#[derive(Clone)]
pub struct FuelWatch {
    default_postcode: i32,
    client: ClientWithMiddleware,
    cache: Cache<&'static str, Arc<Vec<FuelSite>>>,
}

impl FuelWatch {
    pub fn new(settings: &FuelWatchSettings) -> Result<Self, FuelWatchError> {
        let ttl = settings
            .cache_ttl
            .to_std()
            .map_err(|_| FuelWatchError::InvalidCacheTtl)?;

        let cache = Cache::builder().max_capacity(1).time_to_live(ttl).build();

        tracing::info!(
            "fuelwatch integration enabled for {} (cache ttl {ttl:?})",
            settings.postcode
        );

        Ok(Self {
            default_postcode: settings.postcode,
            client: get_traced_http_client()?,
            cache,
        })
    }

    #[instrument(skip(self))]
    pub async fn sites(&self, postcode: Option<i32>) -> Result<Vec<FuelSite>, FuelWatchError> {
        let postcode = postcode.unwrap_or(self.default_postcode);
        let all = self.all_sites().await?;

        let sites: Vec<FuelSite> = all
            .iter()
            .filter(|site| site.postcode == postcode)
            .cloned()
            .collect();

        if sites.is_empty() {
            tracing::warn!("fuelwatch has no unleaded prices for postcode {postcode}");
        }

        Ok(sites)
    }

    async fn all_sites(&self) -> Result<Arc<Vec<FuelSite>>, FuelWatchError> {
        self.cache
            .try_get_with(CACHE_KEY, async {
                tracing::debug!("fuelwatch site cache miss");

                let raw = self.get_sites().await?;

                Ok(Arc::new(shape_sites(raw)))
            })
            .await
            .map_err(FuelWatchError::Shared)
    }

    #[instrument(skip(self))]
    async fn get_sites(&self) -> Result<Vec<Site>, FuelWatchError> {
        let response = self
            .client
            .get(SITES_URL)
            .query(&[("product", PRODUCT_UNLEADED_91)])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(FuelWatchError::Status { status, body });
        }

        Ok(response
            .json()
            .await
            .map_err(reqwest_middleware::Error::from)?)
    }
}

fn shape_sites(raw: Vec<Site>) -> Vec<FuelSite> {
    let mut sites: Vec<FuelSite> = raw
        .into_iter()
        .filter_map(|site| {
            let Some(price) = site.product.price_today else {
                tracing::debug!(
                    "fuelwatch site {} has no price for today; skipping",
                    site.site_name
                );
                return None;
            };

            Some(FuelSite {
                name: site.site_name,
                brand: site.brand_name,
                suburb: site.address.location,
                postcode: site.address.post_code,
                address: site.address.line1,
                price,
                price_tomorrow: site.product.price_tomorrow,
                latitude: site.address.latitude,
                longitude: site.address.longitude,
            })
        })
        .collect();

    sites.sort_by(|a, b| a.price.total_cmp(&b.price));

    sites
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/fuelwatch/sites.json"
    ));

    fn fixture_sites() -> Vec<FuelSite> {
        shape_sites(serde_json::from_str(FIXTURE).unwrap())
    }

    #[test]
    fn shapes_and_sorts_the_upstream_sites() {
        let sites = fixture_sites();

        assert_eq!(sites.len(), 6, "the site without a price for today is gone");

        let cheapest = &sites[0];
        assert_eq!(cheapest.name, "Costco Casuarina");
        assert_eq!(cheapest.brand, "CCO");
        assert_eq!(cheapest.suburb, "CASUARINA");
        assert_eq!(cheapest.postcode, 6167);
        assert_eq!(cheapest.address, "137 Market St");
        assert_eq!(cheapest.price, 186.7);
        assert_eq!(cheapest.price_tomorrow, Some(186.7));

        assert_eq!(sites[1].name, "OTR Yangebup", "sorted cheapest first");
    }

    #[test]
    fn filters_to_a_single_postcode() {
        let sites = fixture_sites();

        let local: Vec<&FuelSite> = sites.iter().filter(|s| s.postcode == 6164).collect();

        assert_eq!(local.len(), 5);
        assert_eq!(local[0].name, "OTR Yangebup");
    }
}
