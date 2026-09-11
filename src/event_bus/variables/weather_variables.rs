use std::collections::BTreeMap;

use crate::event_bus::{ForecastDay, WeatherMetric, WeatherReading, WeatherSource};
use crate::variables::{Node, Shape, Value, VarType};

pub struct WeatherVariables;

impl WeatherVariables {
    pub fn shape(
        source: WeatherSource,
        metric: WeatherMetric,
        day: Option<ForecastDay>,
    ) -> Result<Shape, String> {
        let mut shape = Shape::empty();
        shape.insert("source", Shape::required(VarType::String));

        match source {
            WeatherSource::Bom => {
                for metric in WeatherMetric::BOM {
                    shape.insert(metric.as_str(), Shape::optional(VarType::Float));
                }
            }
            WeatherSource::WillyWeather => {
                for day in ForecastDay::ALL {
                    let mut day_shape = Shape::empty();

                    for metric in WeatherMetric::WILLYWEATHER {
                        day_shape.insert(metric.as_str(), Shape::optional(VarType::Float));
                    }

                    shape.insert(day.as_str(), day_shape);
                }
            }
        }

        let path = match day {
            Some(day) => vec![day.as_str(), metric.as_str()],
            None => vec![metric.as_str()],
        };

        if !shape.require(&path) {
            return Err(format!(
                "weather metric `{}` is not provided by `{}`",
                path.join("."),
                source.as_str()
            ));
        }

        Ok(shape)
    }

    pub fn node(source: WeatherSource, readings: &[WeatherReading]) -> Node {
        let mut node = Node::empty();
        node.insert(
            "source",
            Node::Value(Some(Value::String(source.as_str().to_owned()))),
        );

        let mut days: BTreeMap<&str, Node> = BTreeMap::new();

        for reading in readings {
            let value = Node::Value(Some(Value::Float(reading.value)));

            match reading.day {
                Some(day) => days
                    .entry(day.as_str())
                    .or_insert_with(Node::empty)
                    .insert(reading.metric.as_str(), value),
                None => node.insert(reading.metric.as_str(), value),
            }
        }

        for (day, day_node) in days {
            node.insert(day, day_node);
        }

        node
    }
}
