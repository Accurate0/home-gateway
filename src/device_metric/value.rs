#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Numeric(f64),
    Text(String),
}

impl MetricValue {
    pub fn columns(&self) -> (Option<f64>, Option<&str>) {
        match self {
            MetricValue::Numeric(value) => (Some(*value), None),
            MetricValue::Text(text) => (None, Some(text.as_str())),
        }
    }
}
