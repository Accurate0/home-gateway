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
            step.duration = handle.started.elapsed();

            if step.outcome == StepOutcome::Warning && outcome == StepOutcome::Ran {
                return;
            }

            step.outcome = outcome;
            step.error = error;
        }
    }

    pub fn warn(&self, message: impl Into<String>) {
        let mut steps = self.lock();

        let running = steps
            .iter_mut()
            .rev()
            .find(|step| step.outcome == StepOutcome::Running);

        if let Some(step) = running {
            step.outcome = StepOutcome::Warning;
            step.error = Some(message.into());
        }
    }

    pub fn has_warnings(&self) -> bool {
        self.lock()
            .iter()
            .any(|step| step.outcome == StepOutcome::Warning)
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

    pub fn skipped(
        &self,
        depth: u8,
        kind: impl Into<String>,
        detail: Option<String>,
        guard: String,
    ) {
        self.record(StepTrace {
            depth,
            kind: kind.into(),
            outcome: StepOutcome::GuardSkipped,
            guard: Some(guard),
            detail,
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
        recorder.skipped(0, "light", None, "sun is night".to_owned());
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
    fn a_warning_survives_the_step_finishing_cleanly() {
        let recorder = TraceRecorder::default();

        let outer = recorder.start(0, "lua", None);
        let inner = recorder.start(1, "air_purifier.command", None);
        recorder.warn("purifier is locked");
        recorder.finish(inner, StepOutcome::Ran, None);
        recorder.finish(outer, StepOutcome::Ran, None);

        let steps = recorder.steps();

        assert_eq!(steps[0].outcome, StepOutcome::Ran);
        assert_eq!(steps[1].outcome, StepOutcome::Warning);
        assert_eq!(steps[1].error.as_deref(), Some("purifier is locked"));
        assert!(recorder.has_warnings());
    }

    #[test]
    fn an_error_replaces_a_warning() {
        let recorder = TraceRecorder::default();

        let step = recorder.start(0, "air_purifier", None);
        recorder.warn("purifier is locked");
        recorder.finish(step, StepOutcome::Error, Some("boom".to_owned()));

        assert_eq!(recorder.steps()[0].outcome, StepOutcome::Error);
        assert!(!recorder.has_warnings());
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
