use super::{Node, Shape};

pub trait WorkflowContextVariables {
    fn shape() -> Shape;

    fn to_node(&self) -> Node;
}
