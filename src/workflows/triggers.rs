use crate::integrations::solar::SolarMetric;
use crate::weather::{ForecastDay, WeatherMetric, WeatherReading, WeatherSource};
use crate::workflows::definition::{TriggerMatcher, Workflow};
use std::collections::HashMap;

use crate::event_bus::variables::SolarVariables;
use crate::event_bus::{EventBusMessage, SensorMetric};
use crate::integrations::solar::types::SolarCurrentStatisticsAverages;
use crate::variables::{Node, WorkflowContextVariables};

pub type WeatherKey = (String, WeatherSource, WeatherMetric, Option<ForecastDay>);

#[derive(Default)]
pub struct EdgeLatches {
    /// `(trigger name, sensor, metric) -> comparison satisfied at last reading`.
    /// Lets environment triggers fire on the rising edge only, matching the old
    /// plant-sensor semantics.
    pub last_satisfied: HashMap<(String, String, SensorMetric), bool>,
    /// `(trigger name, metric) -> comparison satisfied at last poll`, the solar
    /// counterpart to `last_satisfied` (there is only one plant, so no subject).
    pub last_solar_satisfied: HashMap<(String, SolarMetric), bool>,
    pub last_weather_satisfied: HashMap<WeatherKey, bool>,
}

impl EdgeLatches {
    pub fn commit(&mut self, latch: Option<PendingLatch>) {
        match latch {
            Some(PendingLatch::Sensor(key)) => {
                self.last_satisfied.insert(key, true);
            }
            Some(PendingLatch::Solar(key)) => {
                self.last_solar_satisfied.insert(key, true);
            }
            Some(PendingLatch::Weather(key)) => {
                self.last_weather_satisfied.insert(key, true);
            }
            None => {}
        }
    }
}

pub fn latch_for(workflow: &Workflow, subject_entity: &str) -> Option<PendingLatch> {
    match &workflow.on {
        TriggerMatcher::Environment { metric, .. } => Some(PendingLatch::Sensor((
            workflow.name.clone(),
            subject_entity.to_owned(),
            metric.clone(),
        ))),
        TriggerMatcher::Solar { metric, .. } => {
            Some(PendingLatch::Solar((workflow.name.clone(), *metric)))
        }
        TriggerMatcher::Weather {
            source,
            metric,
            day,
            ..
        } => Some(PendingLatch::Weather((
            workflow.name.clone(),
            *source,
            *metric,
            *day,
        ))),
        _ => None,
    }
}

pub fn event_node(
    msg: &EventBusMessage,
    averages: Option<&SolarCurrentStatisticsAverages>,
) -> Node {
    match msg {
        EventBusMessage::Solar { current_wh, .. } => {
            SolarVariables::new(*current_wh, averages).to_node()
        }
        other => other.vars(),
    }
}

pub enum PendingLatch {
    Sensor((String, String, SensorMetric)),
    Solar((String, SolarMetric)),
    Weather(WeatherKey),
}

#[allow(clippy::too_many_arguments)]
pub fn weather_fires(
    name: &str,
    source: WeatherSource,
    metric: WeatherMetric,
    day: Option<ForecastDay>,
    cmp: &crate::workflows::definition::Comparison,
    readings: &[WeatherReading],
    last_satisfied: &mut HashMap<WeatherKey, bool>,
    pending: &mut Option<PendingLatch>,
) -> bool {
    let Some(reading) = readings
        .iter()
        .find(|reading| reading.metric == metric && reading.day == day)
    else {
        return false;
    };

    let satisfied = cmp.matches(reading.value);
    let key = (name.to_owned(), source, metric, day);

    if !satisfied {
        last_satisfied.insert(key, false);
        return false;
    }

    if last_satisfied.get(&key).copied().unwrap_or(false) {
        return false;
    }

    *pending = Some(PendingLatch::Weather(key));

    true
}

/// Whether a solar trigger fires for this reading: pick the metric's value
/// (`None` when its average window is still empty, which never fires and leaves
/// the edge state untouched), compare it, and gate on the rising edge so a value
/// sitting past the threshold only fires once.
pub fn solar_fires(
    name: &str,
    metric: SolarMetric,
    cmp: &crate::workflows::definition::Comparison,
    current_wh: f64,
    averages: Option<&SolarCurrentStatisticsAverages>,
    last_satisfied: &mut HashMap<(String, SolarMetric), bool>,
    pending: &mut Option<PendingLatch>,
) -> bool {
    let value = match metric {
        SolarMetric::Current => Some(current_wh),
        SolarMetric::Avg15m => averages.and_then(|a| a.last_15_mins),
        SolarMetric::Avg1h => averages.and_then(|a| a.last_1_hour),
        SolarMetric::Avg3h => averages.and_then(|a| a.last_3_hours),
    };

    let Some(value) = value else {
        return false;
    };

    let satisfied = cmp.matches(value);
    let key = (name.to_owned(), metric);

    if !satisfied {
        last_satisfied.insert(key, false);
        return false;
    }

    if last_satisfied.get(&key).copied().unwrap_or(false) {
        return false;
    }

    *pending = Some(PendingLatch::Solar(key));

    true
}

pub fn unifi_matches(
    clients: Option<&[String]>,
    connected: Option<bool>,
    client: &str,
    mac_address: &str,
    is_connected: bool,
) -> bool {
    let mac = normalise_mac(mac_address);

    connected.is_none_or(|want| want == is_connected)
        && clients.is_none_or(|clients| {
            clients
                .iter()
                .any(|want| want.eq_ignore_ascii_case(client) || normalise_mac(want) == mac)
        })
}

pub fn normalise_mac(mac: &str) -> String {
    mac.to_ascii_lowercase().replace('-', ":")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflows::definition::{CompareOp, Comparison};

    #[test]
    fn unifi_matches_a_listed_client_by_name_ignoring_case() {
        let clients = vec!["pbs".to_owned(), "Raspberry Pi".to_owned()];

        assert!(unifi_matches(
            Some(&clients),
            Some(false),
            "raspberry pi",
            "aa:bb:cc:dd:ee:ff",
            false
        ));
    }

    #[test]
    fn unifi_matches_a_mac_in_a_different_format() {
        let clients = vec!["AA-BB-CC-DD-EE-FF".to_owned()];

        assert!(unifi_matches(
            Some(&clients),
            None,
            "unknown",
            "aa:bb:cc:dd:ee:ff",
            true
        ));
    }

    #[test]
    fn unifi_ignores_unlisted_clients() {
        let clients = vec!["pbs".to_owned()];

        assert!(!unifi_matches(
            Some(&clients),
            Some(false),
            "Living Room TV",
            "11:22:33:44:55:66",
            false
        ));
    }

    #[test]
    fn unifi_filters_on_connected() {
        let clients = vec!["pbs".to_owned()];

        assert!(!unifi_matches(
            Some(&clients),
            Some(false),
            "pbs",
            "aa:bb:cc:dd:ee:ff",
            true
        ));
        assert!(unifi_matches(None, None, "pbs", "aa:bb:cc:dd:ee:ff", true));
    }

    fn averages(last_15_mins: Option<f64>) -> SolarCurrentStatisticsAverages {
        SolarCurrentStatisticsAverages {
            last_15_mins,
            last_1_hour: None,
            last_3_hours: None,
        }
    }

    #[test]
    fn a_sustained_average_fires_once_and_rearms_after_dropping_back() {
        let cmp = Comparison {
            op: CompareOp::Gt,
            value: 3000.0,
        };
        let mut last = HashMap::new();

        let fires = |avg: f64, last: &mut HashMap<_, _>| {
            let mut pending = None;
            let fired = solar_fires(
                "solar surplus",
                SolarMetric::Avg15m,
                &cmp,
                0.0,
                Some(&averages(Some(avg))),
                last,
                &mut pending,
            );

            if let Some(PendingLatch::Solar(key)) = pending {
                last.insert(key, true);
            }

            fired
        };

        assert!(fires(3500.0, &mut last), "expected the crossing to fire");
        assert!(!fires(4000.0, &mut last), "expected no re-fire while past");
        assert!(!fires(2000.0, &mut last), "expected no fire on the drop");
        assert!(fires(3500.0, &mut last), "expected a re-arm and re-fire");
    }

    fn workflow(yaml: &str) -> Workflow {
        serde_yaml::from_str(yaml).expect("workflow yaml")
    }

    #[test]
    fn a_held_threshold_latches_the_edge_it_fired_on() {
        let hot = workflow(
            r#"
name: hot
on: { type: weather, source: bom, metric: temperature, op: gt, value: 35 }
for: 1h
modes: [home]
run: []
"#,
        );

        let mut latches = EdgeLatches::default();
        latches.commit(latch_for(&hot, "bom"));

        assert_eq!(
            latches.last_weather_satisfied.get(&(
                "hot".to_owned(),
                WeatherSource::Bom,
                WeatherMetric::Temperature,
                None
            )),
            Some(&true)
        );
    }

    #[test]
    fn a_held_environment_trigger_latches_on_the_event_sensor() {
        let damp = workflow(
            r#"
name: damp
on: { type: environment, sensor: bathroom, metric: humidity, op: gt, value: 80 }
for: 10m
modes: [home]
run: []
"#,
        );

        assert!(matches!(
            latch_for(&damp, "0xabc"),
            Some(PendingLatch::Sensor((name, sensor, SensorMetric::Humidity)))
                if name == "damp" && sensor == "0xabc"
        ));
    }

    #[test]
    fn presence_holds_have_no_edge_to_latch() {
        let empty = workflow(
            r#"
name: empty
on: { type: presence, sensor: hallway, present: false }
for: 30m
modes: [home]
run: []
"#,
        );

        assert!(latch_for(&empty, "0x1").is_none());
        assert!(empty.on.supports_hold());
    }

    #[test]
    fn a_crossing_rejected_by_a_guard_still_fires_when_the_guard_opens() {
        let cmp = Comparison {
            op: CompareOp::Gt,
            value: 3000.0,
        };
        let mut last = HashMap::new();

        let evaluate = |avg: f64, last: &mut HashMap<_, _>, guard_open: bool| {
            let mut pending = None;
            let fired = solar_fires(
                "solar surplus",
                SolarMetric::Avg15m,
                &cmp,
                0.0,
                Some(&averages(Some(avg))),
                last,
                &mut pending,
            );

            if fired
                && guard_open
                && let Some(PendingLatch::Solar(key)) = pending
            {
                last.insert(key, true);
            }

            fired && guard_open
        };

        assert!(
            !evaluate(3500.0, &mut last, false),
            "the guard is shut, so nothing fires"
        );
        assert!(
            evaluate(3500.0, &mut last, true),
            "the edge was not consumed by the shut guard, so it fires now"
        );
        assert!(
            !evaluate(3500.0, &mut last, true),
            "the edge is consumed once it has fired"
        );
    }

    #[test]
    fn an_empty_average_window_never_fires() {
        let cmp = Comparison {
            op: CompareOp::Lt,
            value: 500.0,
        };
        let mut last = HashMap::new();

        assert!(!solar_fires(
            "low solar",
            SolarMetric::Avg1h,
            &cmp,
            0.0,
            Some(&averages(None)),
            &mut last,
            &mut None,
        ));
        assert!(!solar_fires(
            "low solar",
            SolarMetric::Avg1h,
            &cmp,
            0.0,
            None,
            &mut last,
            &mut None,
        ));
        assert!(last.is_empty(), "edge state should be untouched");
    }

    #[test]
    fn a_weather_forecast_fires_on_the_rising_edge_for_its_day_only() {
        let cmp = Comparison {
            op: CompareOp::Gte,
            value: 35.0,
        };
        let mut last = HashMap::new();

        let forecast = |today: f64, tomorrow: f64| {
            vec![
                WeatherReading {
                    metric: WeatherMetric::MaxTemp,
                    day: Some(ForecastDay::Today),
                    value: today,
                },
                WeatherReading {
                    metric: WeatherMetric::MaxTemp,
                    day: Some(ForecastDay::Tomorrow),
                    value: tomorrow,
                },
            ]
        };

        let fires = |readings: Vec<WeatherReading>, last: &mut HashMap<_, _>| {
            let mut pending = None;
            let fired = weather_fires(
                "hot tomorrow",
                WeatherSource::WillyWeather,
                WeatherMetric::MaxTemp,
                Some(ForecastDay::Tomorrow),
                &cmp,
                &readings,
                last,
                &mut pending,
            );

            if let Some(PendingLatch::Weather(key)) = pending {
                last.insert(key, true);
            }

            fired
        };

        assert!(!fires(forecast(38.0, 30.0), &mut last), "today is ignored");
        assert!(fires(forecast(20.0, 36.0), &mut last), "tomorrow crosses");
        assert!(
            !fires(forecast(20.0, 37.0), &mut last),
            "no re-fire while past"
        );
        assert!(!fires(forecast(20.0, 30.0), &mut last), "drops back");
        assert!(fires(forecast(20.0, 35.0), &mut last), "re-arms");
    }

    #[test]
    fn a_rain_probability_forecast_fires_on_the_rising_edge() {
        let cmp = Comparison {
            op: CompareOp::Gte,
            value: 70.0,
        };
        let mut last = HashMap::new();

        let fires = |probability: f64, last: &mut HashMap<_, _>| {
            let readings = vec![WeatherReading {
                metric: WeatherMetric::RainProbability,
                day: Some(ForecastDay::Tomorrow),
                value: probability,
            }];

            let mut pending = None;
            let fired = weather_fires(
                "rain tomorrow",
                WeatherSource::WillyWeather,
                WeatherMetric::RainProbability,
                Some(ForecastDay::Tomorrow),
                &cmp,
                &readings,
                last,
                &mut pending,
            );

            if let Some(PendingLatch::Weather(key)) = pending {
                last.insert(key, true);
            }

            fired
        };

        assert!(!fires(40.0, &mut last), "below the threshold");
        assert!(fires(80.0, &mut last), "crosses the threshold");
        assert!(!fires(90.0, &mut last), "no re-fire while past");
        assert!(!fires(30.0, &mut last), "drops back");
        assert!(fires(75.0, &mut last), "re-arms");
    }

    #[test]
    fn the_current_metric_reads_the_event_not_the_averages() {
        let cmp = Comparison {
            op: CompareOp::Gt,
            value: 1000.0,
        };
        let mut last = HashMap::new();

        assert!(solar_fires(
            "solar on",
            SolarMetric::Current,
            &cmp,
            1500.0,
            None,
            &mut last,
            &mut None,
        ));
    }
}
