use open_feature::{EvaluationContext, StructValue, Value};

use crate::integrations::feature_flag::FeatureFlagClient;
use crate::tracing_setup::SampleRatios;

pub const TRACING_FLAG: &str = "home-gateway-tracing";

fn ratio(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|n| n as f64))
        .map(|ratio| ratio.clamp(0.0, 1.0))
}

fn ratios_from(value: StructValue, baseline: &SampleRatios) -> SampleRatios {
    let fallback = baseline.clone();

    let default = match value.fields.get("default") {
        Some(field) => ratio(field).unwrap_or_else(|| {
            tracing::warn!(
                "{TRACING_FLAG} default is not a ratio, using {}",
                fallback.default
            );

            fallback.default
        }),
        None => fallback.default,
    };

    let by_span = match value.fields.get("spans") {
        Some(Value::Struct(spans)) => spans
            .fields
            .iter()
            .filter_map(|(span, field)| match ratio(field) {
                Some(ratio) => Some((span.clone(), ratio)),
                None => {
                    tracing::warn!("{TRACING_FLAG} span `{span}` is not a ratio, ignoring");

                    None
                }
            })
            .collect(),
        Some(_) => {
            tracing::warn!("{TRACING_FLAG} spans is not an object, using the defaults");

            fallback.by_span
        }
        None => fallback.by_span,
    };

    SampleRatios { default, by_span }
}

pub async fn evaluate(client: &FeatureFlagClient, baseline: &SampleRatios) -> SampleRatios {
    match client
        .get_struct(TRACING_FLAG, EvaluationContext::default())
        .await
    {
        Ok(value) => ratios_from(value, baseline),
        Err(e) => {
            let fallback = baseline.clone();
            tracing::error!("error evaluating {TRACING_FLAG}: {e:?}, using {fallback:?}");

            fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing_setup::MQTT_INGEST_SPAN;
    use std::collections::HashMap;

    fn baseline() -> SampleRatios {
        SampleRatios {
            default: 1.0,
            by_span: HashMap::from([(MQTT_INGEST_SPAN.to_owned(), 0.05)]),
        }
    }

    fn struct_value(fields: Vec<(&str, Value)>) -> StructValue {
        let mut value = StructValue::default();

        for (key, field) in fields {
            value.fields.insert(key.to_owned(), field);
        }

        value
    }

    fn spans(fields: Vec<(&str, Value)>) -> Value {
        Value::Struct(struct_value(fields))
    }

    #[test]
    fn per_span_ratios_are_read_from_the_flag() {
        let ratios = ratios_from(
            struct_value(vec![
                ("default", Value::Float(1.0)),
                (
                    "spans",
                    spans(vec![
                        ("mqtt.ingest", Value::Float(0.25)),
                        ("dispatch_event", Value::Float(0.0)),
                    ]),
                ),
            ]),
            &baseline(),
        );

        assert_eq!(ratios.default, 1.0);
        assert_eq!(ratios.ratio_for("mqtt.ingest"), 0.25);
        assert_eq!(ratios.ratio_for("dispatch_event"), 0.0);
        assert_eq!(ratios.ratio_for("anything_else"), 1.0);
    }

    #[test]
    fn an_out_of_range_ratio_is_clamped() {
        let ratios = ratios_from(
            struct_value(vec![(
                "spans",
                spans(vec![("mqtt.ingest", Value::Float(7.0))]),
            )]),
            &baseline(),
        );

        assert_eq!(ratios.ratio_for("mqtt.ingest"), 1.0);
    }

    #[test]
    fn an_integer_ratio_is_accepted() {
        let ratios = ratios_from(struct_value(vec![("default", Value::Int(1))]), &baseline());

        assert_eq!(ratios.default, 1.0);
    }

    #[test]
    fn an_empty_flag_keeps_the_configured_baseline() {
        let ratios = ratios_from(struct_value(vec![]), &baseline());

        assert_eq!(ratios, baseline());
        assert_eq!(ratios.ratio_for("mqtt.ingest"), 0.05);
        assert_eq!(ratios.ratio_for("anything_else"), 1.0);
    }

    #[test]
    fn a_wrongly_typed_span_is_ignored_without_dropping_the_rest() {
        let ratios = ratios_from(
            struct_value(vec![(
                "spans",
                spans(vec![
                    ("bad", Value::String("half".to_owned())),
                    ("good", Value::Float(0.5)),
                ]),
            )]),
            &baseline(),
        );

        assert_eq!(ratios.ratio_for("good"), 0.5);
        assert_eq!(ratios.ratio_for("bad"), ratios.default);
    }
}
