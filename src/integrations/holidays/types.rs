use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq)]
pub struct Holiday {
    pub uid: String,
    pub date: NaiveDate,
    pub name: String,
    pub public: bool,
    pub regions: Vec<String>,
}

impl Holiday {
    pub fn observed_in(&self, regions: &[String]) -> bool {
        self.public
            && (self.regions.is_empty()
                || self.regions.iter().any(|region| regions.contains(region)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holiday(public: bool, regions: &[&str]) -> Holiday {
        Holiday {
            uid: "uid".to_owned(),
            date: NaiveDate::from_ymd_opt(2026, 9, 28).unwrap(),
            name: "King's Birthday".to_owned(),
            public,
            regions: regions.iter().map(|region| (*region).to_owned()).collect(),
        }
    }

    #[test]
    fn a_national_public_holiday_is_observed_everywhere() {
        assert!(holiday(true, &[]).observed_in(&["Western Australia".to_owned()]));
    }

    #[test]
    fn a_regional_holiday_is_observed_only_in_its_regions() {
        let wa = ["Western Australia".to_owned()];

        assert!(holiday(true, &["Western Australia"]).observed_in(&wa));
        assert!(!holiday(true, &["Victoria", "Tasmania"]).observed_in(&wa));
    }

    #[test]
    fn an_observance_is_never_a_holiday() {
        assert!(!holiday(false, &[]).observed_in(&["Western Australia".to_owned()]));
    }
}
