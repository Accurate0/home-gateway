const LIPO_CURVE: [(f64, f64); 21] = [
    (4.20, 100.0),
    (4.15, 95.0),
    (4.11, 90.0),
    (4.08, 85.0),
    (4.02, 80.0),
    (3.98, 75.0),
    (3.95, 70.0),
    (3.91, 65.0),
    (3.87, 60.0),
    (3.85, 55.0),
    (3.84, 50.0),
    (3.82, 45.0),
    (3.80, 40.0),
    (3.79, 35.0),
    (3.77, 30.0),
    (3.75, 25.0),
    (3.73, 20.0),
    (3.71, 15.0),
    (3.69, 10.0),
    (3.61, 5.0),
    (3.27, 0.0),
];

pub fn voltage_to_percentage(voltage: f64) -> f64 {
    let (max_v, _) = LIPO_CURVE[0];
    let (min_v, _) = LIPO_CURVE[LIPO_CURVE.len() - 1];

    if voltage >= max_v {
        return 100.0;
    }
    if voltage <= min_v {
        return 0.0;
    }

    for window in LIPO_CURVE.windows(2) {
        let (high_v, high_p) = window[0];
        let (low_v, low_p) = window[1];
        if voltage <= high_v && voltage >= low_v {
            let t = (voltage - low_v) / (high_v - low_v);
            return low_p + t * (high_p - low_p);
        }
    }

    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_and_interpolates() {
        assert_eq!(voltage_to_percentage(4.30), 100.0);
        assert_eq!(voltage_to_percentage(3.20), 0.0);
        assert_eq!(voltage_to_percentage(3.84), 50.0);
        let mid = voltage_to_percentage(4.175);
        assert!((mid - 97.5).abs() < 0.001, "got {mid}");
    }
}
