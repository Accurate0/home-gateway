use crate::variables::{Node, Shape, Value, VarType};

pub struct PlantVariables;

impl PlantVariables {
    pub fn shape() -> Shape {
        let mut shape = Shape::empty();
        shape.insert("sensor", Shape::required(VarType::String));
        shape.insert("soil_moisture", Shape::required(VarType::Float));

        shape
    }

    pub fn node(sensor: &str, soil_moisture: f64) -> Node {
        let mut node = Node::empty();
        node.insert(
            "sensor",
            Node::Value(Some(Value::String(sensor.to_owned()))),
        );
        node.insert(
            "soil_moisture",
            Node::Value(Some(Value::Float(soil_moisture))),
        );

        node
    }
}
