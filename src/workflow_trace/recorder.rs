use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use chrono::Utc;

use super::{StepOutcome, StepTrace};

#[derive(Clone, Default)]
pub struct TraceRecorder {
    steps: Arc<Mutex<Vec<StepTrace>>>,
}

pub struct StepHandle {
    index: usize,
    started: Instant,
}

impl TraceRecorder {
    pub fn start(&self, depth: u8, kind: impl Into<String>, detail: Option<String>) -> StepHandle {
        let mut steps = self.lock();

        steps.push(StepTrace {
            depth,
            kind: kind.into(),
            outcome: StepOutcome::Running,
            guard: None,
            detail,
            error: None,
            duration: Duration::ZERO,
            at: Utc::now(),
        });

        StepHandle {
            index: steps.len() - 1,
            started: Instant::now(),
        }
    }

    pub fn finish(&self, handle: StepHandle, outcome: StepOutcome, error: Option<String>) {
        if let Some(step) = self.lock().get_mut(handle.index) {
            step.outcome = outcome;
            step.error = error;
            step.duration = handle.started.elapsed();
        }
    }

    pub fn record(&self, step: StepTrace) {
        self.lock().push(step);
    }

    pub fn skipped(&self, depth: u8, kind: impl Into<String>, guard: String) {
        self.record(StepTrace {
            depth,
            kind: kind.into(),
            outcome: StepOutcome::GuardSkipped,
            guard: Some(guard),
            detail: None,
            error: None,
            duration: Duration::ZERO,
            at: Utc::now(),
        });
    }

    pub fn steps(&self) -> Vec<StepTrace> {
        self.lock().clone()
    }

    fn lock(&self) -> MutexGuard<'_, Vec<StepTrace>> {
        self.steps
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_steps_keep_their_start_order() {
        let recorder = TraceRecorder::default();

        let scene = recorder.start(0, "scene", None);
        recorder.skipped(0, "light", "sun is night".to_owned());
        let notify = recorder.start(0, "notify", Some("hello".to_owned()));
        recorder.finish(notify, StepOutcome::Error, Some("boom".to_owned()));
        recorder.finish(scene, StepOutcome::Ran, None);

        let steps = recorder.steps();
        let summary = steps
            .iter()
            .map(|s| {
                (
                    s.kind.as_str(),
                    s.outcome,
                    s.guard.as_deref(),
                    s.error.as_deref(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            summary,
            vec![
                ("scene", StepOutcome::Ran, None, None),
                (
                    "light",
                    StepOutcome::GuardSkipped,
                    Some("sun is night"),
                    None
                ),
                ("notify", StepOutcome::Error, None, Some("boom")),
            ]
        );
    }

    #[test]
    fn clones_share_one_trace() {
        let recorder = TraceRecorder::default();
        let clone = recorder.clone();

        let handle = clone.start(1, "lua", None);
        clone.finish(handle, StepOutcome::Ran, None);

        assert_eq!(recorder.steps().len(), 1);
        assert_eq!(recorder.steps()[0].depth, 1);
    }
}
