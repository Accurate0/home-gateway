use super::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    Text(String),
    Expr(Expr),
}
