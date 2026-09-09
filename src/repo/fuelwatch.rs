use sqlx::{Pool, Postgres, Transaction};

use crate::integrations::fuelwatch::types::FuelSite;

#[derive(Clone)]
pub struct FuelWatchRepo {
    db: Pool<Postgres>,
}

struct SiteColumns {
    site_id: Vec<i32>,
    site_name: Vec<String>,
    brand: Vec<String>,
    suburb: Vec<String>,
    postcode: Vec<i32>,
    address: Vec<String>,
    price: Vec<f64>,
    price_tomorrow: Vec<Option<f64>>,
    latitude: Vec<f64>,
    longitude: Vec<f64>,
}

impl SiteColumns {
    fn new(sites: &[FuelSite]) -> Self {
        let mut columns = Self {
            site_id: Vec::with_capacity(sites.len()),
            site_name: Vec::with_capacity(sites.len()),
            brand: Vec::with_capacity(sites.len()),
            suburb: Vec::with_capacity(sites.len()),
            postcode: Vec::with_capacity(sites.len()),
            address: Vec::with_capacity(sites.len()),
            price: Vec::with_capacity(sites.len()),
            price_tomorrow: Vec::with_capacity(sites.len()),
            latitude: Vec::with_capacity(sites.len()),
            longitude: Vec::with_capacity(sites.len()),
        };

        for site in sites {
            columns.site_id.push(site.site_id);
            columns.site_name.push(site.name.clone());
            columns.brand.push(site.brand.clone());
            columns.suburb.push(site.suburb.clone());
            columns.postcode.push(site.postcode);
            columns.address.push(site.address.clone());
            columns.price.push(site.price);
            columns.price_tomorrow.push(site.price_tomorrow);
            columns.latitude.push(site.latitude);
            columns.longitude.push(site.longitude);
        }

        columns
    }
}

impl FuelWatchRepo {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }

    #[tracing::instrument(skip_all, name = "db.fuelwatch.replace_sites", fields(sites = sites.len()), err)]
    pub async fn replace_sites(
        &self,
        tx: &mut Transaction<'static, Postgres>,
        sites: &[FuelSite],
    ) -> Result<u64, sqlx::Error> {
        sqlx::query!("DELETE FROM fuelwatch_site")
            .execute(&mut **tx)
            .await?;

        if sites.is_empty() {
            return Ok(0);
        }

        let columns = SiteColumns::new(sites);

        let inserted = sqlx::query!(
            r#"
            INSERT INTO fuelwatch_site (
                site_id, site_name, brand, suburb, postcode, address,
                price, price_tomorrow, latitude, longitude
            )
            SELECT * FROM UNNEST(
                $1::INTEGER[], $2::TEXT[], $3::TEXT[], $4::TEXT[], $5::INTEGER[], $6::TEXT[],
                $7::DOUBLE PRECISION[], $8::DOUBLE PRECISION[],
                $9::DOUBLE PRECISION[], $10::DOUBLE PRECISION[]
            )
            ON CONFLICT (site_id) DO UPDATE SET
                site_name = EXCLUDED.site_name,
                brand = EXCLUDED.brand,
                suburb = EXCLUDED.suburb,
                postcode = EXCLUDED.postcode,
                address = EXCLUDED.address,
                price = EXCLUDED.price,
                price_tomorrow = EXCLUDED.price_tomorrow,
                latitude = EXCLUDED.latitude,
                longitude = EXCLUDED.longitude,
                updated_at = now()
            "#,
            &columns.site_id,
            &columns.site_name,
            &columns.brand,
            &columns.suburb,
            &columns.postcode,
            &columns.address,
            &columns.price,
            &columns.price_tomorrow as &[Option<f64>],
            &columns.latitude,
            &columns.longitude,
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();

        Ok(inserted)
    }

    #[tracing::instrument(skip_all, name = "db.fuelwatch.append_history", fields(sites = sites.len()), err)]
    pub async fn append_history(
        &self,
        tx: &mut Transaction<'static, Postgres>,
        sites: &[FuelSite],
    ) -> Result<u64, sqlx::Error> {
        if sites.is_empty() {
            return Ok(0);
        }

        let columns = SiteColumns::new(sites);

        let inserted = sqlx::query!(
            r#"
            INSERT INTO fuelwatch_price_history (
                site_id, postcode, price, price_tomorrow
            )
            SELECT * FROM UNNEST(
                $1::INTEGER[], $2::INTEGER[],
                $3::DOUBLE PRECISION[], $4::DOUBLE PRECISION[]
            )
            "#,
            &columns.site_id,
            &columns.postcode,
            &columns.price,
            &columns.price_tomorrow as &[Option<f64>],
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();

        Ok(inserted)
    }

    #[tracing::instrument(skip_all, name = "db.fuelwatch.sites_for_postcode", fields(postcode), err)]
    pub async fn sites_for_postcode(
        &self,
        postcode: i32,
        limit: Option<i64>,
    ) -> Result<Vec<FuelSite>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT site_id, site_name, brand, suburb, postcode, address,
                   price, price_tomorrow, latitude, longitude
            FROM fuelwatch_site
            WHERE postcode = $1
            ORDER BY price ASC
            LIMIT $2
            "#,
            postcode,
            limit,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| FuelSite {
                site_id: row.site_id,
                name: row.site_name,
                brand: row.brand,
                suburb: row.suburb,
                postcode: row.postcode,
                address: row.address,
                price: row.price,
                price_tomorrow: row.price_tomorrow,
                latitude: row.latitude,
                longitude: row.longitude,
            })
            .collect())
    }
}
