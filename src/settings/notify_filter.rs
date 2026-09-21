use wildmatch::WildMatch;

#[derive(Debug, Clone, Default)]
pub struct NotifyFilter {
    disabled: Vec<WildMatch>,
}

impl NotifyFilter {
    pub fn new<S: AsRef<str>>(patterns: &[S]) -> Self {
        Self {
            disabled: patterns
                .iter()
                .map(|pattern| WildMatch::new(pattern.as_ref()))
                .collect(),
        }
    }

    pub fn is_disabled(&self, source: &str) -> bool {
        self.disabled.iter().any(|pattern| pattern.matches(source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wildcard_matches_a_whole_group() {
        let filter = NotifyFilter::new(&["watchdog.*"]);

        assert!(filter.is_disabled("watchdog.stale"));
        assert!(filter.is_disabled("watchdog.recovered"));
        assert!(!filter.is_disabled("door.left_open"));
    }

    #[test]
    fn an_exact_id_does_not_match_a_longer_one() {
        let filter = NotifyFilter::new(&["workflow.notify.x"]);

        assert!(filter.is_disabled("workflow.notify.x"));
        assert!(!filter.is_disabled("workflow.notify.xy"));
    }

    #[test]
    fn a_star_disables_everything() {
        let filter = NotifyFilter::new(&["*"]);

        assert!(filter.is_disabled("graphql.push"));
    }

    #[test]
    fn an_empty_filter_disables_nothing() {
        assert!(!NotifyFilter::default().is_disabled("watchdog.stale"));
    }
}
