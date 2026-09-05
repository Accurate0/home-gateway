use axum::Json;
use axum::extract::State;
use http::StatusCode;
use serde::Serialize;

use crate::actors::health::ActorHealthRegistry;
use crate::actors::health::Lifecycle;
use crate::actors::manifest;
use crate::state::AppState;

pub async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

#[derive(Serialize)]
pub struct ActorStatus {
    name: &'static str,
    lifecycle: Lifecycle,
    expected: bool,
    present: bool,
}

#[derive(Serialize)]
pub struct ActorHealth {
    healthy: bool,
    actors: Vec<ActorStatus>,
    registered: Vec<String>,
}

fn is_expected(name: &str, lifecycle: Lifecycle) -> bool {
    match lifecycle {
        Lifecycle::Skipped => false,
        Lifecycle::Running => true,
        Lifecycle::Unavailable => manifest::find(name).is_some_and(|spec| !spec.optional),
    }
}

pub async fn actor_health(State(state): State<AppState>) -> (StatusCode, Json<ActorHealth>) {
    let actors: Vec<ActorStatus> = state
        .handles
        .expect::<ActorHealthRegistry>()
        .snapshot()
        .into_iter()
        .map(|(name, lifecycle)| ActorStatus {
            name,
            lifecycle,
            expected: is_expected(name, lifecycle),
            present: ractor::registry::where_is(name).is_some(),
        })
        .collect();

    let healthy = actors.iter().all(|a| !a.expected || a.present);
    let status = if healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(ActorHealth {
            healthy,
            actors,
            registered: ractor::registry::registered(),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::system::adhoc::AdhocTaskActor;
    use crate::actors::workflows::WorkflowWorker;

    #[test]
    fn a_skipped_actor_is_not_expected() {
        assert!(!is_expected(WorkflowWorker::NAME, Lifecycle::Skipped));
    }

    #[test]
    fn a_running_actor_is_expected() {
        assert!(is_expected(WorkflowWorker::NAME, Lifecycle::Running));
    }

    #[test]
    fn an_unavailable_optional_actor_does_not_fail_the_check() {
        assert!(!is_expected(AdhocTaskActor::NAME, Lifecycle::Unavailable));
        assert!(is_expected(WorkflowWorker::NAME, Lifecycle::Unavailable));
    }
}
