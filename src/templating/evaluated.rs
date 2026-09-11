use crate::variables::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Evaluated {
    pub value: Option<Value>,
    pub decimals: Option<u32>,
}

impl Evaluated {
    pub fn render(&self) -> Option<String> {
        let value = self.value.as_ref()?;

        let rendered = match (value, self.decimals) {
            (Value::Float(number), Some(decimals)) => {
                format!("{number:.precision$}", precision = decimals as usize)
            }
            (value, _) => value.render(),
        };

        Some(rendered)
    }
}
