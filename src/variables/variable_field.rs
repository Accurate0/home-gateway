use super::{Node, ScalarVariable, Shape};

pub trait VariableField {
    fn field_shape() -> Shape;

    fn field_node(&self) -> Node;
}

impl<T: ScalarVariable> VariableField for T {
    fn field_shape() -> Shape {
        Shape::required(T::TYPE)
    }

    fn field_node(&self) -> Node {
        Node::Value(Some(self.to_value()))
    }
}

impl<T: ScalarVariable> VariableField for Option<T> {
    fn field_shape() -> Shape {
        Shape::optional(T::TYPE)
    }

    fn field_node(&self) -> Node {
        Node::Value(self.as_ref().map(ScalarVariable::to_value))
    }
}
