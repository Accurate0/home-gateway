use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    Running,
    Ran,
    GuardSkipped,
    DryRun,
    Error,
}

impl fmt::Display for StepOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            StepOutcome::Running => "running",
            StepOutcome::Ran => "ran",
            StepOutcome::GuardSkipped => "guard_skipped",
            StepOutcome::DryRun => "dry_run",
            StepOutcome::Error => "error",
        };

        f.write_str(name)
    }
}
