use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use chrono::Utc;

use super::{StepOutcome, StepTrace};

#[derive(Clone, Default)]
pub struct TraceRecorder {
    steps: Arc<Mutex<Vec<StepTrace>>>,
    nesting: Arc<AtomicU8>,
}

pub struct StepHandle {
    index: usize,
    started: Instant,
}

pub struct NestingGuard {
    nesting: Arc<AtomicU8>,
}

impl Drop for NestingGuard {
    fn drop(&mut self) {
        self.nesting.fetch_sub(1, Ordering::Relaxed);
    }
}

impl TraceRecorder {
    pub fn start(&self, depth: u8, kind: impl Into<String>, detail: Option<String>) -> StepHandle {
        let depth = self.nested(depth);
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

    pub fn finish_with(&self, handle: StepHandle, error: Option<String>) {
        let outcome = if error.is_some() {
            StepOutcome::Error
        } else {
            StepOutcome::Ran
        };

        self.finish(handle, outcome, error);
    }

    pub fn record(&self, step: StepTrace) {
        let depth = self.nested(step.depth);

        self.lock().push(StepTrace { depth, ..step });
    }

    pub fn nest(&self) -> NestingGuard {
        self.nesting.fetch_add(1, Ordering::Relaxed);

        NestingGuard {
            nesting: self.nesting.clone(),
        }
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

    fn nested(&self, depth: u8) -> u8 {
        depth.saturating_add(self.nesting.load(Ordering::Relaxed))
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

    #[test]
    fn nesting_indents_steps_until_the_guard_drops() {
        let recorder = TraceRecorder::default();

        let outer = recorder.start(1, "fetch", None);
        let guard = recorder.nest();
        let inner = recorder.start(1, "gw.http", None);
        recorder.finish(inner, StepOutcome::Ran, None);
        drop(guard);
        recorder.finish(outer, StepOutcome::Ran, None);
        let after = recorder.start(1, "light.set", None);
        recorder.finish(after, StepOutcome::Ran, None);

        let depths = recorder
            .steps()
            .iter()
            .map(|s| (s.kind.clone(), s.depth))
            .collect::<Vec<_>>();

        assert_eq!(
            depths,
            vec![
                ("fetch".to_owned(), 1),
                ("gw.http".to_owned(), 2),
                ("light.set".to_owned(), 1),
            ]
        );
    }
}
