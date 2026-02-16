use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorType {
    Current,
    Voltage,
    Power,
    Energy,
    Frequency,
    Temperature,
    Pressure,
    StateOfCharge,
    Other,
}

#[derive(Debug, Clone)]
pub struct Reading {
    pub sensor_type: SensorType,
    pub unit: String,
    pub value: f64,
    pub timestamp_ms: u64,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
pub trait Driver: Send + Sync {
    /// Unique identifier for this driver instance (e.g., "modbus-inverter-1")
    fn id(&self) -> &str;
    
    /// Connect to the device (establishes TCP/Serial link)
    async fn connect(&mut self) -> anyhow::Result<()>;
    
    /// Poll all configured datapoints
    async fn read_all(&mut self) -> anyhow::Result<Vec<Reading>>;
    
    /// Close connection
    async fn disconnect(&mut self) -> anyhow::Result<()>;
}
