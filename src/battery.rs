use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, async_graphql::Enum, sqlx::Type,
)]
#[serde(rename_all = "snake_case")]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum BatteryChemistry {
    Unknown,
    Lipo,
    LiIon,
    Alkaline,
    Cr2032,
}

impl BatteryChemistry {
    fn range(self) -> Option<(f64, f64)> {
        match self {
            Self::Unknown => None,
            Self::Lipo => Some((3.3, 4.2)),
            Self::LiIon => Some((3.0, 4.2)),
            Self::Alkaline => Some((0.9, 1.5)),
            Self::Cr2032 => Some((2.0, 3.0)),
        }
    }
}

pub fn voltage_to_percentage(chemistry: BatteryChemistry, voltage: f64) -> Option<f64> {
    let (min_v, max_v) = chemistry.range()?;

    Some(((voltage - min_v) * 100.0 / (max_v - min_v)).clamp(0.0, 100.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(chemistry: BatteryChemistry, voltage: f64, expected: f64) {
        let p = voltage_to_percentage(chemistry, voltage).unwrap();
        assert!(
            (p - expected).abs() < 1e-6,
            "{chemistry:?} {voltage} -> {p}"
        );
    }

    #[test]
    fn maps_endpoints_and_midpoint() {
        approx(BatteryChemistry::Lipo, 3.3, 0.0);
        approx(BatteryChemistry::Lipo, 4.2, 100.0);
        approx(BatteryChemistry::Lipo, 3.75, 50.0);

        approx(BatteryChemistry::LiIon, 3.6, 50.0);
        approx(BatteryChemistry::Alkaline, 1.2, 50.0);
        approx(BatteryChemistry::Cr2032, 2.5, 50.0);
    }

    #[test]
    fn clamps_outside_range() {
        assert_eq!(
            voltage_to_percentage(BatteryChemistry::Lipo, 4.30),
            Some(100.0)
        );
        assert_eq!(
            voltage_to_percentage(BatteryChemistry::Lipo, 3.20),
            Some(0.0)
        );
        assert_eq!(
            voltage_to_percentage(BatteryChemistry::Cr2032, 0.0),
            Some(0.0)
        );
        assert_eq!(
            voltage_to_percentage(BatteryChemistry::Alkaline, 9.0),
            Some(100.0)
        );
    }

    #[test]
    fn unknown_chemistry_has_no_percentage() {
        assert_eq!(voltage_to_percentage(BatteryChemistry::Unknown, 3.75), None);
    }

    #[test]
    fn interpolates_linearly() {
        let p = voltage_to_percentage(BatteryChemistry::Lipo, 3.95).unwrap();
        assert!((p - 72.222).abs() < 0.001, "got {p}");
    }

    #[test]
    fn deserializes_firmware_chemistry_strings() {
        let parsed: BatteryChemistry = serde_json::from_str("\"lipo\"").unwrap();
        assert_eq!(parsed, BatteryChemistry::Lipo);

        let parsed: BatteryChemistry = serde_json::from_str("\"li_ion\"").unwrap();
        assert_eq!(parsed, BatteryChemistry::LiIon);

        let parsed: BatteryChemistry = serde_json::from_str("\"unknown\"").unwrap();
        assert_eq!(parsed, BatteryChemistry::Unknown);
    }
}
