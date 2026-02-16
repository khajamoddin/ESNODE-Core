use agent_core::drivers::{Driver, Reading, SensorType};
use async_trait::async_trait;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use std::io::BufReader;
use std::fs::File;
use rumqttc::Transport;
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile::{certs, pkcs8_private_keys};
use rustls_native_certs::load_native_certs;

/// Configuration for MQTT driver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    /// MQTT broker address (e.g., "mqtt.example.com")
    pub broker: String,
    /// MQTT broker port (default: 1883)
    pub port: u16,
    /// Client ID for MQTT connection
    pub client_id: String,
    /// Optional username for authentication
    pub username: Option<String>,
    /// Optional password for authentication
    pub password: Option<String>,
    /// Topics to subscribe to (supports wildcards: +, #)
    pub topics: Vec<String>,
    /// QoS level (0, 1, or 2)
    pub qos: u8,
    /// Enable TLS/SSL
    #[serde(default)]
    pub use_tls: bool,
    /// Path to CA certificate file (for TLS)
    pub ca_cert_path: Option<String>,
    /// Path to client certificate file (for mTLS)  
    pub client_cert_path: Option<String>,
    /// Path to client private key file (for mTLS)
    pub client_key_path: Option<String>,
    /// Topic-to-sensor mappings
    pub topic_mappings: Vec<TopicMapping>,
}

/// Maps an MQTT topic to sensor metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicMapping {
    /// Topic pattern (can use wildcards)
    pub topic: String,
    /// Sensor type for this topic (as string: "temperature", "pressure", etc.)
    pub sensor_type_str: String,
    /// Unit of measurement
    pub unit: String,
    /// JSON path to value (e.g., "temperature", "data.value")
    pub value_path: String,
    /// Optional scale factor
    #[serde(default = "default_scale")]
    pub scale: f64,
}

impl TopicMapping {
    /// Convert string sensor type to SensorType enum
    pub fn sensor_type(&self) -> SensorType {
        match self.sensor_type_str.to_lowercase().as_str() {
            "temperature" => SensorType::Temperature,
            "pressure" => SensorType::Pressure,
            "voltage" => SensorType::Voltage,
            "current" => SensorType::Current,
            "power" => SensorType::Power,
            "energy" => SensorType::Energy,
            "frequency" => SensorType::Frequency,
            "stateofcharge" | "soc" => SensorType::StateOfCharge,
            _ => SensorType::Other,
        }
    }
    
    /// Create new mapping with SensorType
    pub fn new(topic: String, sensor_type: SensorType, unit: String, value_path: String, scale: f64) -> Self {
        let sensor_type_str = match sensor_type {
            SensorType::Temperature => "temperature",
            SensorType::Pressure => "pressure",
            SensorType::Voltage => "voltage",
            SensorType::Current => "current",
            SensorType::Power => "power",
            SensorType::Energy => "energy",
            SensorType::Frequency => "frequency",
            SensorType::StateOfCharge => "soc",
            SensorType::Other => "other",
        }.to_string();
        
        Self {
            topic,
            sensor_type_str,
            unit,
            value_path,
            scale,
        }
    }
}

fn default_scale() -> f64 {
    1.0
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker: "localhost".to_string(),
            port: 1883,
            client_id: "esnode-mqtt".to_string(),
            username: None,
            password: None,
            topics: vec!["sensors/#".to_string()],
            qos: 1,
            use_tls: false,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            topic_mappings: vec![TopicMapping::new(
                "sensors/temperature".to_string(),
                SensorType::Temperature,
                "celsius".to_string(),
                "value".to_string(),
                1.0,
            )],
        }
    }
}

/// MQTT Driver for subscribing to IoT sensor data
pub struct MqttDriver {
    id: String,
    config: MqttConfig,
    client: Option<AsyncClient>,
    readings_buffer: Arc<Mutex<Vec<Reading>>>,
}

impl MqttDriver {
    pub fn new(id: String, config: MqttConfig) -> Self {
        Self {
            id,
            config,
            client: None,
            readings_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Parse JSON payload and extract value using JSON path
    fn extract_value(&self, payload: &str, value_path: &str) -> Option<f64> {
        let json: serde_json::Value = serde_json::from_str(payload).ok()?;
        
        // Simple JSON path traversal (supports "key" or "key.subkey")
        let parts: Vec<&str> = value_path.split('.').collect();
        let mut current = &json;
        
        for part in parts {
            current = current.get(part)?;
        }
        
        // Try to extract as number
        match current {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::String(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    /// Match topic to mapping
    fn find_mapping(&self, topic: &str) -> Option<&TopicMapping> {
        self.config.topic_mappings.iter().find(|mapping| {
            // Simple wildcard matching (MQTT style)
            Self::topic_matches(&mapping.topic, topic)
        })
    }

    /// Simple MQTT wildcard matching
    fn topic_matches(pattern: &str, topic: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let topic_parts: Vec<&str> = topic.split('/').collect();
        
        if pattern_parts.len() != topic_parts.len() && !pattern.contains('#') {
            return false;
        }
        
        for (i, pattern_part) in pattern_parts.iter().enumerate() {
            if *pattern_part == "#" {
                return true; // Multi-level wildcard matches everything after
            }
            if *pattern_part == "+" {
                continue; // Single-level wildcard matches any single level
            }
            if i >= topic_parts.len() || *pattern_part != topic_parts[i] {
                return false;
            }
        }
        
        true
    }

    /// Spawn background task to receive MQTT messages
    async fn spawn_receiver(&self, mut eventloop: rumqttc::EventLoop) {
        let buffer = self.readings_buffer.clone();
        let config = self.config.clone();
        let driver_id = self.id.clone();

        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Incoming::Publish(publish))) => {
                        let topic = publish.topic.clone();
                        let payload = String::from_utf8_lossy(&publish.payload).to_string();
                        
                        tracing::debug!("MQTT message received: topic={}, payload={}", topic, payload);

                        // Find matching topic mapping
                        if let Some(mapping) = Self::find_mapping_static(&config, &topic) {
                            if let Some(value) = Self::extract_value_static(&payload, &mapping.value_path) {
                                let scaled_value = value * mapping.scale;
                                
                                let mut metadata = HashMap::new();
                                metadata.insert("topic".to_string(), topic.clone());
                                metadata.insert("driver_id".to_string(), driver_id.clone());
                                
                                let reading = Reading {
                                    sensor_type: mapping.sensor_type(),
                                    unit: mapping.unit.clone(),
                                    value: scaled_value,
                                    timestamp_ms: SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis() as u64,
                                    metadata,
                                };
                                
                                let mut buf = buffer.lock().await;
                                buf.push(reading);
                                
                                // Keep buffer size limited (last 1000 readings)
                                if buf.len() > 1000 {
                                    buf.drain(0..500);
                                }
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("MQTT eventloop error: {:?}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });
    }

    // Static helpers for use in async block
    fn find_mapping_static(config: &MqttConfig, topic: &str) -> Option<TopicMapping> {
        config.topic_mappings.iter().find(|mapping| {
            Self::topic_matches(&mapping.topic, topic)
        }).cloned()
    }

    fn extract_value_static(payload: &str, value_path: &str) -> Option<f64> {
        let json: serde_json::Value = serde_json::from_str(payload).ok()?;
        
        let parts: Vec<&str> = value_path.split('.').collect();
        let mut current = &json;
        
        for part in parts {
            current = current.get(part)?;
        }
        
        match current {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::String(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }
}

#[async_trait]
impl Driver for MqttDriver {
    fn id(&self) -> &str {
        &self.id
    }

    async fn connect(&mut self) -> anyhow::Result<()> {
        let mut mqtt_options = MqttOptions::new(
            &self.config.client_id,
            &self.config.broker,
            self.config.port,
        );

        mqtt_options.set_keep_alive(std::time::Duration::from_secs(30));

        if let Some(username) = &self.config.username {
            let password = self.config.password.clone().unwrap_or_default();
            mqtt_options.set_credentials(username, password);
        }

        if self.config.use_tls {
            tracing::info!("Configuring TLS for MQTT connection");

            // Load CA certificate if provided, otherwise use system certs
            let mut root_cert_store = RootCertStore::empty();
            if let Some(ca_path) = &self.config.ca_cert_path {
                let mut reader = BufReader::new(File::open(ca_path)?);
                for cert in certs(&mut reader) {
                    root_cert_store.add(cert?)?;
                }
            } else {
                // Use system certificates
                for cert in load_native_certs()? {
                    root_cert_store.add(rustls::pki_types::CertificateDer::from(cert))?;
                }
            };
            
            // Build TLS config
            let builder = ClientConfig::builder()
                .with_root_certificates(root_cert_store);

            // Add client certificate if provided (mTLS)
            let tls_config = if let (Some(cert_path), Some(key_path)) = 
                (&self.config.client_cert_path, &self.config.client_key_path) 
            {
                tracing::info!("Configuring mTLS with client certificate: {}", cert_path);
                let certs = {
                    let mut reader = BufReader::new(File::open(cert_path)?);
                    certs(&mut reader)
                        .collect::<Result<Vec<_>, _>>()?
                };
                let key = {
                    let mut reader = BufReader::new(File::open(key_path)?);
                    let keys = pkcs8_private_keys(&mut reader)
                        .collect::<Result<Vec<_>, _>>()?;
                    if keys.is_empty() {
                        anyhow::bail!("No PKCS8 private keys found in {}", key_path);
                    }
                    rustls::pki_types::PrivateKeyDer::from(keys[0].clone_key())
                };
                builder.with_client_auth_cert(certs, key)?
            } else {
                builder.with_no_client_auth()
            };

            mqtt_options.set_transport(Transport::tls_with_config(tls_config.into()));
            tracing::info!("TLS configured successfully");
        }
        let (client, eventloop) = AsyncClient::new(mqtt_options, 100);

        // Subscribe to all configured topics
        let qos = match self.config.qos {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => QoS::AtLeastOnce,
        };

        for topic in &self.config.topics {
            client.subscribe(topic, qos).await?;
            tracing::info!("MQTT subscribed to topic: {}", topic);
        }

        // Spawn background receiver
        self.spawn_receiver(eventloop).await;

        self.client = Some(client);
        Ok(())
    }

    async fn read_all(&mut self) -> anyhow::Result<Vec<Reading>> {
        // Drain the readings buffer
        let mut buffer = self.readings_buffer.lock().await;
        let readings = buffer.drain(..).collect();
        Ok(readings)
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        if let Some(client) = &self.client {
            // Unsubscribe from all topics
            for topic in &self.config.topics {
                let _ = client.unsubscribe(topic).await;
            }
        }
        self.client = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_matching() {
        assert!(MqttDriver::topic_matches("sensors/+/temperature", "sensors/room1/temperature"));
        assert!(MqttDriver::topic_matches("sensors/#", "sensors/room1/temperature"));
        assert!(MqttDriver::topic_matches("sensors/#", "sensors/room1/temperature/value"));
        assert!(!MqttDriver::topic_matches("sensors/+/temperature", "sensors/room1/humidity"));
        assert!(MqttDriver::topic_matches("sensors/room1/temperature", "sensors/room1/temperature"));
    }

    #[test]
    fn test_json_extraction() {
        let driver = MqttDriver::new(
            "test".to_string(),
            MqttConfig::default(),
        );

        let json1 = r#"{"value": 23.5}"#;
        assert_eq!(driver.extract_value(json1, "value"), Some(23.5));

        let json2 = r#"{"data": {"temperature": 25.0}}"#;
        assert_eq!(driver.extract_value(json2, "data.temperature"), Some(25.0));

        let json3 = r#"{"reading": "42.3"}"#;
        assert_eq!(driver.extract_value(json3, "reading"), Some(42.3));
    }

    #[tokio::test]
    async fn test_mqtt_driver_lifecycle() {
        let config = MqttConfig {
            broker: "test.mosquitto.org".to_string(),
            port: 1883,
            client_id: "esnode-test".to_string(),
            username: None,
            password: None,
            topics: vec!["esnode/test".to_string()],
            qos: 0,
            use_tls: false,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            topic_mappings: vec![],
        };

        let mut driver = MqttDriver::new("test-mqtt".to_string(), config);

        // Note: This will fail if test.mosquitto.org is unreachable
        // In production, use a local MQTT broker for tests
        match driver.connect().await {
            Ok(_) => {
                tracing::info!("Connected to MQTT broker");
                assert!(driver.client.is_some());
                
                // Test disconnect
                driver.disconnect().await.unwrap();
                assert!(driver.client.is_none());
            }
            Err(e) => {
                tracing::warn!("Could not connect to public MQTT broker: {:?}", e);
                // This is acceptable in CI environments without internet
            }
        }
    }
}
