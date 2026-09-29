#[derive(Debug, Default)]
pub struct Reachability {
    reported: Option<bool>,
}

impl Reachability {
    pub fn changed(&mut self, connected: bool) -> bool {
        self.reported.replace(connected) != Some(connected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_change_in_reachability_is_reported() {
        let mut reachability = Reachability::default();

        assert!(reachability.changed(false));
        assert!(!reachability.changed(false));
        assert!(reachability.changed(true));
        assert!(!reachability.changed(true));
        assert!(reachability.changed(false));
    }

    #[test]
    fn a_device_that_never_connects_is_reported_once() {
        let mut reachability = Reachability::default();

        assert!(
            reachability.changed(false),
            "a device that is down at startup should be reported"
        );
        assert!(
            !reachability.changed(false),
            "every reconnect attempt should not re-report it"
        );
    }
}
