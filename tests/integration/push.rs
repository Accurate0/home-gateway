use chrono::TimeDelta;
use home_gateway::repo::notification_interaction::NotificationInteraction;
use home_gateway::repo::push::{NewPushNotification, PushRepo};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::common::db::fresh_database;

fn bins(remind_after: TimeDelta, reminders: u32) -> NewPushNotification<'static> {
    NewPushNotification {
        tag: "workflow:bins",
        title: "Home Gateway",
        body: "Bins",
        category: "general",
        actions: serde_json::json!([{ "label": "Done", "type": "acknowledge" }]),
        remind_after,
        reminders,
    }
}

#[tokio::test]
async fn an_unacknowledged_notification_is_reminded_once_then_stops() {
    let repo = PushRepo::new(fresh_database().await.pool);

    let sent = repo.record_sent(bins(TimeDelta::zero(), 1)).await.unwrap();

    assert_eq!(sent.send_count, 1);
    assert!(sent.next_reminder_at.is_some());

    let reminded = repo.take_due_reminder(sent.id).await.unwrap().unwrap();

    assert_eq!(reminded.send_count, 2);
    assert_eq!(reminded.next_reminder_at, None);
    assert!(repo.take_due_reminder(sent.id).await.unwrap().is_none());
    assert!(repo.pending_reminders().await.unwrap().is_empty());
}

#[tokio::test]
async fn acknowledging_before_the_reminder_is_due_cancels_it() {
    let repo = PushRepo::new(fresh_database().await.pool);

    let sent = repo.record_sent(bins(TimeDelta::zero(), 1)).await.unwrap();

    let recorded = repo
        .record_interaction(sent.id, NotificationInteraction::Acknowledged)
        .await
        .unwrap();

    assert!(recorded);
    assert!(repo.take_due_reminder(sent.id).await.unwrap().is_none());

    let latest = repo.recent_notifications(1).await.unwrap().remove(0);

    assert_eq!(latest.send_count, 1);
    assert!(latest.acknowledged_at.is_some());
    assert_eq!(latest.next_reminder_at, None);
}

#[tokio::test]
async fn other_interactions_are_tracked_without_cancelling_the_reminder() {
    let repo = PushRepo::new(fresh_database().await.pool);

    let sent = repo
        .record_sent(bins(TimeDelta::hours(2), 1))
        .await
        .unwrap();

    for kind in [
        NotificationInteraction::Opened,
        NotificationInteraction::Swiped,
    ] {
        assert!(repo.record_interaction(sent.id, kind).await.unwrap());
    }

    let interactions = repo.interactions(&[sent.id]).await.unwrap();
    let kinds: Vec<&str> = interactions.iter().map(|i| i.kind.as_str()).collect();

    assert_eq!(kinds, vec!["opened", "swiped"]);
    assert_eq!(repo.pending_reminders().await.unwrap().len(), 1);
    assert!(repo.take_due_reminder(sent.id).await.unwrap().is_none());
}

#[tokio::test]
async fn a_notification_without_reminders_is_never_scheduled() {
    let repo = PushRepo::new(fresh_database().await.pool);

    let sent = repo
        .record_sent(bins(TimeDelta::hours(2), 0))
        .await
        .unwrap();

    assert_eq!(sent.next_reminder_at, None);
    assert!(repo.pending_reminders().await.unwrap().is_empty());
}

#[tokio::test]
async fn an_interaction_for_an_unknown_notification_is_rejected() {
    let repo = PushRepo::new(fresh_database().await.pool);

    let recorded = repo
        .record_interaction(Uuid::new_v4(), NotificationInteraction::Acknowledged)
        .await
        .unwrap();

    assert!(!recorded);
}
