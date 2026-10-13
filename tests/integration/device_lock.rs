use chrono::{TimeDelta, Utc};
use home_gateway::device_command::CommandOrigin;
use home_gateway::device_lock::{DeviceLocks, LockSettings, LockedDevice};
use home_gateway::event_bus::{EventBus, EventBusMessage};
use home_gateway::intent::Attribution;
use home_gateway::repo::RepoRegistry;
use home_gateway::repo::air_purifier::{AirPurifierMode, AirPurifierState};
use home_gateway::repo::device_lock::DeviceLockSource;
use home_gateway::repo::intent::{
    AirPurifierAttributes, DeviceKind, DeviceReport, IntentAttributes,
};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::common::db::fresh_database;

const DEVICE: &str = "purifier-core300s-1";
const ADDRESS: &str = "purifier.iot";

fn settings(excluded: Vec<&'static str>) -> LockSettings {
    LockSettings {
        physical: TimeDelta::hours(6),
        excluded,
    }
}

fn state(mode: AirPurifierMode, display: bool) -> AirPurifierState {
    AirPurifierState {
        on: true,
        mode: Some(mode),
        speed: Some(1),
        display: Some(display),
    }
}

async fn observe(
    locks: &DeviceLocks<'_>,
    settings: &LockSettings,
    previous: AirPurifierState,
    current: AirPurifierState,
) -> Attribution {
    locks
        .observe(
            &LockedDevice {
                device_id: DEVICE,
                address: ADDRESS,
                settings,
            },
            Some(&DeviceReport::AirPurifier(&previous)),
            &DeviceReport::AirPurifier(&current),
        )
        .await
}

#[tokio::test]
async fn a_change_nobody_asked_for_locks_the_device_against_workflows() {
    let repos = RepoRegistry::new(fresh_database().await.pool);
    let event_bus = EventBus::default();
    let mut events = event_bus.subscribe();
    let locks = DeviceLocks::new(&repos, &event_bus);

    let attribution = observe(
        &locks,
        &settings(vec!["display"]),
        state(AirPurifierMode::Auto, true),
        state(AirPurifierMode::Sleep, true),
    )
    .await;

    assert_eq!(
        attribution,
        Attribution::External {
            fields: vec!["mode"]
        }
    );

    let lock = repos.device_lock().active(DEVICE).await.unwrap().unwrap();

    assert_eq!(lock.source, DeviceLockSource::Physical);
    assert!(lock.expires_at.unwrap() > Utc::now() + TimeDelta::hours(5));

    let blocked = locks.guard(CommandOrigin::Workflow, DEVICE).await.unwrap();
    let allowed = locks.guard(CommandOrigin::User, DEVICE).await.unwrap();

    assert!(blocked.is_some());
    assert!(allowed.is_none());

    assert!(matches!(
        events.try_recv().unwrap(),
        EventBusMessage::DeviceLock { locked: true, .. }
    ));
}

#[tokio::test]
async fn an_excluded_change_does_not_lock() {
    let repos = RepoRegistry::new(fresh_database().await.pool);
    let event_bus = EventBus::default();
    let locks = DeviceLocks::new(&repos, &event_bus);

    observe(
        &locks,
        &settings(vec!["display"]),
        state(AirPurifierMode::Auto, true),
        state(AirPurifierMode::Auto, false),
    )
    .await;

    assert_eq!(repos.device_lock().active(DEVICE).await.unwrap(), None);
}

#[tokio::test]
async fn a_change_the_gateway_asked_for_does_not_lock_and_confirms_the_intent() {
    let repos = RepoRegistry::new(fresh_database().await.pool);
    let event_bus = EventBus::default();
    let locks = DeviceLocks::new(&repos, &event_bus);

    let wanted = IntentAttributes::AirPurifier(AirPurifierAttributes {
        on: Some(true),
        mode: Some(AirPurifierMode::Sleep),
        ..AirPurifierAttributes::default()
    });

    repos
        .intent()
        .replace_pending(ADDRESS, &wanted, false, Uuid::new_v4())
        .await
        .unwrap();

    let attribution = observe(
        &locks,
        &settings(Vec::new()),
        state(AirPurifierMode::Auto, true),
        state(AirPurifierMode::Sleep, true),
    )
    .await;

    let pending = repos
        .intent()
        .pending_for(DeviceKind::AirPurifier, ADDRESS)
        .await
        .unwrap();

    assert_eq!(attribution, Attribution::Gateway);
    assert_eq!(repos.device_lock().active(DEVICE).await.unwrap(), None);
    assert!(pending.is_empty());
}

#[tokio::test]
async fn a_physical_change_supersedes_what_the_gateway_still_wanted() {
    let repos = RepoRegistry::new(fresh_database().await.pool);
    let event_bus = EventBus::default();
    let locks = DeviceLocks::new(&repos, &event_bus);

    let wanted = IntentAttributes::AirPurifier(AirPurifierAttributes {
        on: Some(true),
        mode: Some(AirPurifierMode::Sleep),
        ..AirPurifierAttributes::default()
    });

    repos
        .intent()
        .replace_pending(ADDRESS, &wanted, false, Uuid::new_v4())
        .await
        .unwrap();

    observe(
        &locks,
        &settings(Vec::new()),
        state(AirPurifierMode::Auto, true),
        state(AirPurifierMode::Manual, true),
    )
    .await;

    let pending = repos
        .intent()
        .pending_for(DeviceKind::AirPurifier, ADDRESS)
        .await
        .unwrap();

    assert!(pending.is_empty());
    assert!(repos.device_lock().active(DEVICE).await.unwrap().is_some());
}

#[tokio::test]
async fn a_physical_change_never_shortens_a_manual_lock() {
    let repos = RepoRegistry::new(fresh_database().await.pool);
    let event_bus = EventBus::default();
    let locks = DeviceLocks::new(&repos, &event_bus);

    locks.lock(DEVICE, None).await.unwrap();

    observe(
        &locks,
        &settings(Vec::new()),
        state(AirPurifierMode::Auto, true),
        state(AirPurifierMode::Sleep, true),
    )
    .await;

    let lock = repos.device_lock().active(DEVICE).await.unwrap().unwrap();

    assert_eq!(lock.source, DeviceLockSource::Manual);
    assert_eq!(lock.expires_at, None);
}

#[tokio::test]
async fn a_physical_change_outlasts_a_shorter_manual_lock() {
    let repos = RepoRegistry::new(fresh_database().await.pool);
    let event_bus = EventBus::default();
    let locks = DeviceLocks::new(&repos, &event_bus);

    locks
        .lock(DEVICE, Some(Utc::now() + TimeDelta::hours(1)))
        .await
        .unwrap();

    observe(
        &locks,
        &settings(Vec::new()),
        state(AirPurifierMode::Auto, true),
        state(AirPurifierMode::Sleep, true),
    )
    .await;

    let lock = repos.device_lock().active(DEVICE).await.unwrap().unwrap();

    assert_eq!(lock.source, DeviceLockSource::Physical);
}

#[tokio::test]
async fn an_expired_lock_blocks_nothing() {
    let repos = RepoRegistry::new(fresh_database().await.pool);
    let event_bus = EventBus::default();
    let locks = DeviceLocks::new(&repos, &event_bus);

    locks
        .lock(DEVICE, Some(Utc::now() - TimeDelta::seconds(1)))
        .await
        .unwrap();

    let blocked = locks.guard(CommandOrigin::Workflow, DEVICE).await.unwrap();

    assert_eq!(blocked, None);
}

#[tokio::test]
async fn unlocking_lets_workflows_through_again() {
    let repos = RepoRegistry::new(fresh_database().await.pool);
    let event_bus = EventBus::default();
    let locks = DeviceLocks::new(&repos, &event_bus);

    locks.lock(DEVICE, None).await.unwrap();

    let removed = locks.unlock(DEVICE).await.unwrap();
    let again = locks.unlock(DEVICE).await.unwrap();
    let blocked = locks.guard(CommandOrigin::Workflow, DEVICE).await.unwrap();

    assert!(removed);
    assert!(!again);
    assert_eq!(blocked, None);
}
