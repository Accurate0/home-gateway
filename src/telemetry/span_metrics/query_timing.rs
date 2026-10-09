use std::time::{Duration, Instant};

pub struct QueryTiming {
    started: Instant,
    failed: bool,
}

impl QueryTiming {
    pub fn start() -> Self {
        QueryTiming {
            started: Instant::now(),
            failed: false,
        }
    }

    pub fn fail(&mut self) {
        self.failed = true;
    }

    pub fn outcome(&self) -> &'static str {
        if self.failed { "error" } else { "success" }
    }

    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::QueryTiming;

    #[test]
    fn a_query_succeeds_until_it_is_failed() {
        let mut timing = QueryTiming::start();

        assert_eq!(timing.outcome(), "success");

        timing.fail();

        assert_eq!(timing.outcome(), "error");
    }
}
