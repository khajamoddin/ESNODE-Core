use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub timestamp_ms: u64,
    pub labels: Vec<(String, String)>,
}

impl Metric {
    pub fn new(name: &str, value: f64, timestamp_ms: u64) -> Self {
        Self {
            name: name.to_string(),
            value,
            timestamp_ms,
            labels: Vec::new(),
        }
    }

    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.labels.push((key.to_string(), value.to_string()));
        self
    }
}
