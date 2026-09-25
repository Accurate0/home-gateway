use chrono::DateTime;
use home_gateway::actors::integrations::synergy::interval::EnergyInterval;
use home_gateway::lua::{LuaCallContext, LuaError};
use serial_test::serial;
use uuid::Uuid;

use crate::common::Harness;

const EXPORT: &str = "Date,Time,ANYTIME (KWH),Solar export (Units),Billing Status\r\n\
    26/06/2026,00:00,0.190,0.000,Billed\r\n\
    23/09/2026,23:30,0.185,1.250,Not yet billed\r\n";

async fn parse(harness: &Harness, csv: &str) -> Result<Vec<EnergyInterval>, LuaError> {
    let cx = LuaCallContext::new(harness.state.clone(), Uuid::new_v4(), "synergy");

    harness
        .state
        .lua
        .call_integration(
            &cx,
            &harness.state.settings.integrations.synergy.parser,
            &[serde_json::Value::String(csv.to_owned())],
        )
        .await
}

#[tokio::test]
#[serial]
async fn an_interval_export_parses_into_perth_local_intervals() {
    let harness = Harness::start().await;

    let intervals = parse(&harness, EXPORT)
        .await
        .expect("the export should parse");

    assert_eq!(intervals.len(), 2);
    assert_eq!(intervals[0].used, 0.190);
    assert_eq!(intervals[1].exported, 1.250);
    assert_eq!(
        intervals[1].at,
        DateTime::parse_from_rfc3339("2026-09-23T23:30:00+08:00")
            .unwrap()
            .timestamp()
    );
}

#[tokio::test]
#[serial]
async fn an_export_missing_a_column_is_rejected() {
    let harness = Harness::start().await;

    let error = parse(&harness, "Date,Time,Usage\n01/01/2026,00:00,1\n")
        .await
        .expect_err("a missing column should fail the upload");

    assert!(
        error
            .to_string()
            .contains("missing `ANYTIME (KWH)`, `Solar export (Units)`"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
#[serial]
async fn a_malformed_interval_is_rejected() {
    let harness = Harness::start().await;

    let error = parse(
        &harness,
        "Date,Time,ANYTIME (KWH),Solar export (Units),Billing Status\n01/01/2026,00:00,x,0,Billed\n",
    )
    .await
    .expect_err("a malformed row should fail the upload");

    assert!(
        error.to_string().contains("line 2"),
        "unexpected error: {error}"
    );
}
