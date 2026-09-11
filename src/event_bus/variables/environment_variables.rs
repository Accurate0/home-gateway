use crate::event_bus::{SensorMetric, SensorReading};
use crate::variables::{Node, Shape, Value, VarType};

pub struct EnvironmentVariables;

impl EnvironmentVariables {
    pub fn shape(metrics: &[SensorMetric], trigger: &SensorMetric) -> Result<Shape, String> {
        if !metrics.contains(trigger) {
            let reported = metrics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");

            return Err(format!(
                "sensor does not report `{trigger}`; it reports: [{reported}]"
            ));
        }

        let mut shape = Shape::empty();
        shape.insert("sensor", Shape::required(VarType::String));

        for metric in metrics {
            shape.insert(metric.to_string(), Shape::optional(VarType::Float));
        }

        shape.require(&[trigger.to_string().as_str()]);

        Ok(shape)
    }

    pub fn node(sensor: &str, readings: &[SensorReading]) -> Node {
        let mut node = Node::empty();
        node.insert(
            "sensor",
            Node::Value(Some(Value::String(sensor.to_owned()))),
        );

        for reading in readings {
            node.insert(
                reading.metric().to_string(),
                Node::Value(Some(Value::Float(reading.value()))),
            );
        }

        node
    }
}
