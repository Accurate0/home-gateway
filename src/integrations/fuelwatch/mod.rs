use reqwest_middleware::ClientWithMiddleware;
use tracing::instrument;

use crate::http::get_traced_http_client;
use crate::integrations::fuelwatch::types::{FuelSite, Site};
use crate::settings::FuelWatchSettings;

pub mod types;
pub mod variables;

const SITES_URL: &str = "https://www.fuelwatch.wa.gov.au/api/sites";
const PRODUCT_UNLEADED_91: &str = "1";

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
}

#[derive(Clone)]
pub struct FuelWatch {
    client: ClientWithMiddleware,
}

impl FuelWatch {
    pub fn new(
        settings: &FuelWatchSettings,
        timeout: std::time::Duration,
    ) -> Result<Self, FuelWatchError> {
        tracing::info!("fuelwatch integration enabled for {}", settings.postcode);

        Ok(Self {
            client: get_traced_http_client(timeout)?,
        })
    }

    #[instrument(skip(self), err)]
    pub async fn fetch_sites(&self) -> Result<Vec<FuelSite>, FuelWatchError> {
        Ok(shape_sites(self.get_sites().await?))
    }

    #[instrument(skip(self))]
    async fn get_sites(&self) -> Result<Vec<Site>, FuelWatchError> {
        let response = self
            .client
            .get(SITES_URL)
            .with_extension(crate::http::UrlTemplate("/api/sites"))
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
                site_id: site.id,
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
