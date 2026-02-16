use crate::collectors::Collector;
use crate::drivers::Driver;
use crate::metrics::MetricsRegistry;
use crate::state::StatusState;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

pub struct ProtocolRunner {
    drivers: Arc<Mutex<Vec<Box<dyn Driver>>>>,
    status: StatusState,
}

impl ProtocolRunner {
    pub fn new(drivers: Vec<Box<dyn Driver>>, status: StatusState) -> Self {
        for d in &drivers {
            info!("Protocol Runner: Loaded driver {}", d.id());
        }
        Self {
            drivers: Arc::new(Mutex::new(drivers)),
            status,
        }
    }
}

#[async_trait]
impl Collector for ProtocolRunner {
    fn name(&self) -> &'static str {
        "protocol_runner"
    }

    async fn collect(&mut self, metrics: &MetricsRegistry) -> anyhow::Result<()> {
        let mut drivers = self.drivers.lock().await;

        for driver in drivers.iter_mut() {
            match driver.read_all().await {
                Ok(readings) => {
                    for reading in readings {
                        // Export reading to Prometheus
                        // We need a generic metric in MetricsRegistry for this.
                        // For now, we use a gauge Vec labeled by "driver", "sensor_type", "unit", and any metadata.
                        
                        // Note: To support high cardinality or dynamic labels, we might need a dedicated metric family.
                        // Reusing an existing one or adding a new one in metrics.rs is best.
                        
                        let sensor_type_str = format!("{:?}", reading.sensor_type);
                        
                        // Using a generic 'iot_sensor_value' gauge
                        // Labels: driver_id, sensor_type, unit, param (from metadata if any)
                        
                        let param = reading.metadata.get("register")
                            .or_else(|| reading.metadata.get("oid"))
                            .map(|s| s.as_str())
                            .unwrap_or("unknown");

                        metrics.iot_sensor_value
                            .with_label_values(&[
                                driver.id(),
                                &sensor_type_str,
                                &reading.unit,
                                param
                            ])
                            .set(reading.value);
                    }
                }
                Err(e) => {
                    error!("Driver {} failed: {:?}", driver.id(), e);
                    // Attempt reconnect?
                    // Ideally, the driver internal logic handles reconnect on next read_all or we call connect here.
                    // For now, simpler to just log.
                    // If we want self-healing:
                    let _ = driver.connect().await;
                }
            }
        }
        Ok(())
    }
}
