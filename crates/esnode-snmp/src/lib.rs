use agent_core::drivers::{Driver, Reading, SensorType};
use async_trait::async_trait;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SnmpConfig {
    pub target: SocketAddr,
    pub community: String,
    pub oids: Vec<String>,
}

pub struct SnmpDriver {
    pub id: String,
    pub config: SnmpConfig,
    socket: Option<Arc<UdpSocket>>,
}

impl SnmpDriver {
    pub fn new(id: String, config: SnmpConfig) -> Self {
        Self {
            id,
            config,
            socket: None,
        }
    }
}

#[async_trait]
impl Driver for SnmpDriver {
    fn id(&self) -> &str {
        &self.id
    }

    async fn connect(&mut self) -> anyhow::Result<()> {
        // Bind to a random local port
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(self.config.target).await?;
        self.socket = Some(Arc::new(socket));
        Ok(())
    }

    async fn read_all(&mut self) -> anyhow::Result<Vec<Reading>> {
        let mut readings = Vec::new();
        
        if let Some(socket) = &self.socket {
            for oid in &self.config.oids {
                // Construct a minimal SNMP GetRequest packet (Simulated)
                // Version: 1 (0x00)
                // Community: public
                // PDU: GetRequest
                
                // For MVP, sending a dummy payload to trigger traffic
                let payload = format!("GET {}", oid).into_bytes();
                socket.send(&payload).await?;
                
                // Receive response
                let mut buf = [0u8; 1024];
                // Use timeout for UDP receive
                let res = tokio::time::timeout(std::time::Duration::from_millis(100), socket.recv(&mut buf)).await;
                
                match res {
                    Ok(Ok(n)) => {
                        // Simulate parsing response
                        // Real implementation would decode specific ASN.1 type
                        if n > 0 {
                            readings.push(Reading {
                                sensor_type: SensorType::Other,
                                unit: "raw".to_string(),
                                value: n as f64, // Just return byte count as value for now
                                timestamp_ms: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis() as u64,
                                metadata: {
                                    let mut m = HashMap::new();
                                    m.insert("oid".to_string(), oid.clone());
                                    m
                                }
                            });
                        }
                    }
                    Ok(Err(e)) => return Err(anyhow::anyhow!("UDP Recv Error: {:?}", e)),
                    Err(_) => {
                        // Timeout is common in UDP if device is offline
                        // Log warning but continue
                    }
                }
            }
        }
        
        Ok(readings)
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        self.socket = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_snmp_udp() {
        // Start Mock UDP Server
        let server_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr = server_socket.local_addr().unwrap();
        
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                // Echo server
                let (len, remote_addr) = server_socket.recv_from(&mut buf).await.unwrap();
                server_socket.send_to(&buf[..len], remote_addr).await.unwrap();
            }
        });

        let config = SnmpConfig {
            target: addr,
            community: "public".to_string(),
            oids: vec!["1.3.6.1.2.1.1.1.0".to_string()],
        };

        let mut driver = SnmpDriver::new("test-snmp".to_string(), config);
        
        driver.connect().await.unwrap();
        let readings = driver.read_all().await.unwrap();
        
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].metadata.get("oid").unwrap(), "1.3.6.1.2.1.1.1.0");
        
        driver.disconnect().await.unwrap();
    }
}
