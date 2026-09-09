use home_gateway::integrations::fuelwatch::types::FuelSite;
use home_gateway::repo::fuelwatch::FuelWatchRepo;
use pretty_assertions::assert_eq;

use crate::common::db::fresh_database;

fn site(id: i32, name: &str, postcode: i32, price: f64) -> FuelSite {
    FuelSite {
        site_id: id,
        name: name.to_owned(),
        brand: "BRAND".to_owned(),
        suburb: "YANGEBUP".to_owned(),
        postcode,
        address: format!("{name} road"),
        price,
        price_tomorrow: Some(price + 1.0),
        latitude: -32.1,
        longitude: 115.8,
    }
}

#[tokio::test]
async fn sites_are_returned_cheapest_first_for_one_postcode() {
    let db = fresh_database().await;
    let repo = FuelWatchRepo::new(db.pool.clone());

    let sites = vec![
        site(1, "dearest local", 6164, 199.9),
        site(2, "cheapest local", 6164, 186.7),
        site(3, "middle local", 6164, 190.0),
        site(4, "elsewhere", 6167, 150.0),
    ];

    let mut tx = db.pool.begin().await.expect("begin");
    let stored = repo.replace_sites(&mut tx, &sites).await.expect("replace");
    tx.commit().await.expect("commit");

    assert_eq!(stored, 4);

    let local = repo
        .sites_for_postcode(6164, None)
        .await
        .expect("read local sites");

    let names: Vec<&str> = local.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["cheapest local", "middle local", "dearest local"],
        "the cheaper postcode 6167 site must not leak in, and order is by price"
    );
}

#[tokio::test]
async fn a_limit_is_applied_in_the_query() {
    let db = fresh_database().await;
    let repo = FuelWatchRepo::new(db.pool.clone());

    let sites = vec![
        site(1, "cheapest", 6164, 180.0),
        site(2, "second", 6164, 181.0),
        site(3, "third", 6164, 182.0),
    ];

    let mut tx = db.pool.begin().await.expect("begin");
    repo.replace_sites(&mut tx, &sites).await.expect("replace");
    tx.commit().await.expect("commit");

    let limited = repo
        .sites_for_postcode(6164, Some(1))
        .await
        .expect("read limited");

    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].name, "cheapest");
}

#[tokio::test]
async fn a_refresh_removes_sites_that_left_the_upstream_feed() {
    let db = fresh_database().await;
    let repo = FuelWatchRepo::new(db.pool.clone());

    let mut tx = db.pool.begin().await.expect("begin");
    repo.replace_sites(&mut tx, &[site(1, "closing down", 6164, 190.0)])
        .await
        .expect("first replace");
    tx.commit().await.expect("commit");

    let mut tx = db.pool.begin().await.expect("begin");
    repo.replace_sites(&mut tx, &[site(2, "still trading", 6164, 191.0)])
        .await
        .expect("second replace");
    tx.commit().await.expect("commit");

    let current = repo
        .sites_for_postcode(6164, None)
        .await
        .expect("read current");

    let names: Vec<&str> = current.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["still trading"],
        "wholesale replacement must drop sites absent from the new batch"
    );
}

#[tokio::test]
async fn history_appends_a_row_per_site_per_refresh() {
    let db = fresh_database().await;
    let repo = FuelWatchRepo::new(db.pool.clone());

    let sites = vec![site(1, "one", 6164, 180.0), site(2, "two", 6164, 181.0)];

    for _ in 0..2 {
        let mut tx = db.pool.begin().await.expect("begin");
        repo.append_history(&mut tx, &sites)
            .await
            .expect("append history");
        tx.commit().await.expect("commit");
    }

    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM fuelwatch_price_history")
        .fetch_one(&db.pool)
        .await
        .expect("count history");

    assert_eq!(rows, 4, "history is append-only, unlike the current table");
}

#[tokio::test]
async fn an_empty_batch_leaves_history_untouched() {
    let db = fresh_database().await;
    let repo = FuelWatchRepo::new(db.pool.clone());

    let mut tx = db.pool.begin().await.expect("begin");
    let appended = repo
        .append_history(&mut tx, &[])
        .await
        .expect("append none");
    tx.commit().await.expect("commit");

    assert_eq!(appended, 0);
}
