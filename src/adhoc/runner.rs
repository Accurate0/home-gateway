use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};

use futures::FutureExt;
use open_feature::EvaluationContext;

use super::context::AdhocTaskContext;
use super::cron_task::AdhocCronTask;
use super::error::AdhocTaskError;
use super::queries::{read_ledger, record_cron_run, write_ledger};
use super::registry;
use super::task::{AdhocTask, checksum};
use crate::state::SharedActorState;

const TASK_TIMEOUT: Duration = Duration::from_secs(300);

enum Step {
    Continue,
    Park,
}

#[derive(Debug, PartialEq, Eq)]
enum Ledger {
    Completed,
    Pending,
    Tampered,
}

fn classify(existing: Option<(&str, &str)>, name: &str, expected: &str) -> Ledger {
    match existing {
        None => Ledger::Pending,
        Some((row_name, row_checksum)) if row_name == name && row_checksum == expected => {
            Ledger::Completed
        }
        Some(_) => Ledger::Tampered,
    }
}

pub async fn run_pending(state: &SharedActorState) {
    for task in registry() {
        match run_one(state, task).await {
            Step::Continue => continue,
            Step::Park => {
                tracing::warn!(
                    "adhoc queue parked at ordinal {} ({})",
                    task.ordinal(),
                    task.name()
                );

                return;
            }
        }
    }

    tracing::debug!("adhoc queue is fully drained");
}

async fn run_one(state: &SharedActorState, task: &'static dyn AdhocTask) -> Step {
    let expected = checksum(task.source());

    let row = match read_ledger(&state.db, task.ordinal()).await {
        Ok(row) => row,
        Err(e) => {
            tracing::error!("failed reading adhoc ledger for {}: {e}", task.name());
            crate::metrics::record_adhoc_task(task.name(), "ledger_error");

            return Step::Park;
        }
    };

    let ledger = classify(
        row.as_ref()
            .map(|row| (row.name.as_str(), row.checksum.as_str())),
        task.name(),
        &expected,
    );

    match ledger {
        Ledger::Completed => {
            tracing::trace!("adhoc task {} already completed, skipping", task.name());

            return Step::Continue;
        }
        Ledger::Tampered => {
            let row = row.expect("tampered implies a ledger row");
            tracing::error!(
                "adhoc task {} (ordinal {}) was edited after running: ledger has name={} checksum={}, code has checksum={}",
                task.name(),
                task.ordinal(),
                row.name,
                row.checksum,
                expected
            );
            crate::metrics::record_adhoc_task(task.name(), "checksum_mismatch");

            return Step::Park;
        }
        Ledger::Pending => {
            tracing::debug!("adhoc task {} is pending", task.name());
        }
    }

    if !is_released(state, task).await {
        tracing::info!("adhoc task {} is held by its rollout flag", task.name());
        crate::metrics::record_adhoc_task(task.name(), "held");

        return Step::Park;
    }

    match execute(state, task).await {
        Ok(elapsed) => {
            tracing::info!(
                "adhoc task {} completed in {}ms",
                task.name(),
                elapsed.as_millis()
            );
            crate::metrics::record_adhoc_task(task.name(), "success");

            Step::Continue
        }
        Err(e) => {
            tracing::error!("adhoc task {} failed and was rolled back: {e}", task.name());
            crate::metrics::record_adhoc_task(task.name(), "error");

            Step::Park
        }
    }
}

async fn is_released(state: &SharedActorState, task: &'static dyn AdhocTask) -> bool {
    flag_released(state, task.name(), task.flag()).await
}

async fn flag_released(
    state: &SharedActorState,
    name: &'static str,
    flag: Option<&'static str>,
) -> bool {
    let Some(flag) = flag else {
        tracing::trace!("adhoc task {name} has no rollout flag");

        return true;
    };

    if !state.feature_flag_client.has_live_provider() {
        tracing::debug!("no live flag provider, treating adhoc task {name} as released");

        return true;
    }

    state
        .feature_flag_client
        .is_feature_enabled(flag, true, EvaluationContext::default())
        .await
}

pub async fn run_cron(state: &SharedActorState, task: &'static dyn AdhocCronTask) {
    if !flag_released(state, task.name(), task.flag()).await {
        tracing::info!("adhoc cron task {} is held by its flag", task.name());
        crate::metrics::record_adhoc_task(task.name(), "held");

        return;
    }

    let start = Instant::now();

    match execute_cron(state, task).await {
        Ok(rows) => {
            let elapsed = start.elapsed();
            tracing::info!(
                "adhoc cron task {} affected {rows} rows in {}ms",
                task.name(),
                elapsed.as_millis()
            );
            crate::metrics::record_adhoc_task(task.name(), "success");
            save_cron_run(state, task.name(), elapsed, rows as i64, "success").await;
        }
        Err(e) => {
            let elapsed = start.elapsed();
            tracing::error!(
                "adhoc cron task {} failed and was rolled back: {e}",
                task.name()
            );
            crate::metrics::record_adhoc_task(task.name(), "error");
            save_cron_run(state, task.name(), elapsed, 0, "error").await;
        }
    }
}

async fn execute_cron(
    state: &SharedActorState,
    task: &'static dyn AdhocCronTask,
) -> Result<u64, AdhocTaskError> {
    let mut tx = state.db.begin().await?;

    let rows = {
        let mut ctx = AdhocTaskContext::new(state, &mut tx);

        match tokio::time::timeout(
            TASK_TIMEOUT,
            AssertUnwindSafe(task.run(&mut ctx)).catch_unwind(),
        )
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(AdhocTaskError::Panicked),
            Err(_) => Err(AdhocTaskError::TimedOut),
        }
    }?;

    tx.commit().await?;

    Ok(rows)
}

async fn save_cron_run(
    state: &SharedActorState,
    name: &'static str,
    elapsed: Duration,
    rows: i64,
    outcome: &'static str,
) {
    let recorded =
        record_cron_run(&state.db, name, elapsed.as_millis() as i64, rows, outcome).await;

    if let Err(e) = recorded {
        tracing::error!("failed recording adhoc cron run for {name}: {e}");
    }
}

async fn execute(
    state: &SharedActorState,
    task: &'static dyn AdhocTask,
) -> Result<Duration, AdhocTaskError> {
    let start = Instant::now();
    let mut tx = state.db.begin().await?;

    let result = {
        let mut ctx = AdhocTaskContext::new(state, &mut tx);

        match tokio::time::timeout(
            TASK_TIMEOUT,
            AssertUnwindSafe(task.run(&mut ctx)).catch_unwind(),
        )
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(AdhocTaskError::Panicked),
            Err(_) => Err(AdhocTaskError::TimedOut),
        }
    };

    result?;

    let elapsed = start.elapsed();

    write_ledger(
        &mut tx,
        task.ordinal(),
        task.name(),
        &checksum(task.source()),
        elapsed.as_millis() as i64,
    )
    .await?;

    tx.commit().await?;

    Ok(elapsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn missing_row_is_pending() {
        assert_eq!(classify(None, "trim", "abc"), Ledger::Pending);
    }

    #[test]
    fn matching_row_is_completed() {
        assert_eq!(
            classify(Some(("trim", "abc")), "trim", "abc"),
            Ledger::Completed
        );
    }

    #[test]
    fn changed_checksum_is_tampered() {
        assert_eq!(
            classify(Some(("trim", "old")), "trim", "abc"),
            Ledger::Tampered
        );
    }

    #[test]
    fn reused_ordinal_under_new_name_is_tampered() {
        assert_eq!(
            classify(Some(("other", "abc")), "trim", "abc"),
            Ledger::Tampered
        );
    }
}
