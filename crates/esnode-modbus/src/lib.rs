use agent_core::drivers::{Driver, Reading, SensorType};
use async_trait::async_trait;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_modbus::client::{Client, Context, Reader, tcp};
use tokio_modbus::prelude::SlaveContext;
use tokio_modbus::Slave;

#[derive(Debug, Clone)]
pub struct RegisterMapping {
    pub address: u16,
    pub count: u16,
    pub sensor_type: SensorType,
    pub unit: String,
    pub scale: f64, // Multiplier (e.g., 0.1 for 1 decimal)
}

pub struct ModbusDriver {
    pub id: String,
    pub addr: SocketAddr,
    pub slave_id: Slave,
    pub mappings: Vec<RegisterMapping>,
    // Wrap in Mutex to satisfy Sync requirement of Driver trait
    ctx: Option<Arc<Mutex<Context>>>,
}

impl ModbusDriver {
    pub fn new(id: String, addr: SocketAddr, slave_id: u8, mappings: Vec<RegisterMapping>) -> Self {
        Self {
            id,
            addr,
            slave_id: Slave(slave_id),
            mappings,
            ctx: None,
        }
    }
}

#[async_trait]
impl Driver for ModbusDriver {
    fn id(&self) -> &str {
        &self.id
    }

    async fn connect(&mut self) -> anyhow::Result<()> {
        let mut ctx = tcp::connect(self.addr).await?;
        ctx.set_slave(self.slave_id);
        self.ctx = Some(Arc::new(Mutex::new(ctx)));
        Ok(())
    }

    async fn read_all(&mut self) -> anyhow::Result<Vec<Reading>> {
        let mut readings = Vec::new();
        
        if let Some(ctx_mutex) = &self.ctx {
            let mut ctx = ctx_mutex.lock().await;
            
            for map in &self.mappings {
                // Read Input Registers (Function Code 04)
                // Handle IO error then Exception code
                let response = ctx.read_input_registers(map.address, map.count).await
                    .map_err(|e| anyhow::anyhow!("Modbus IO Error at {}: {:?}", map.address, e))?;
                
                let data = response.map_err(|e| anyhow::anyhow!("Modbus Exception at {}: {:?}", map.address, e))?;
                
                let raw_value = if map.count == 2 {
                    let high = data[0] as u32;
                    let low = data[1] as u32;
                    (high << 16 | low) as f64
                } else {
                    data[0] as f64
                };
                
                let value = raw_value * map.scale;
                
                let mut metadata = HashMap::new();
                metadata.insert("register".to_string(), map.address.to_string());
                
                readings.push(Reading {
                    sensor_type: map.sensor_type,
                    unit: map.unit.clone(),
                    value,
                    timestamp_ms: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis() as u64,
                    metadata,
                });
            }
        } else {
            return Err(anyhow::anyhow!("Not connected"));
        }
        
        Ok(readings)
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        if let Some(ctx_mutex) = &self.ctx {
             let mut ctx = ctx_mutex.lock().await;
             ctx.disconnect().await?;
        }
        self.ctx = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_modbus_read() {
        // Start Mock Server (Raw TCP)
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        tokio::spawn(async move {
             let (mut socket, _) = listener.accept().await.unwrap();
             let mut buf = [0u8; 1024];
             loop {
                 let n = socket.read(&mut buf).await.unwrap();
                 if n == 0 { break; }
                 
                 // Simple mock: assume it's a valid ReadInputRegisters request
                 // Request: TransId(2), ProtoId(2), Len(2), Unit(1), Fun(1), Addr(2), Cnt(2)
                 // Response: TransId(2), ProtoId(2), Len(2), Unit(1), Fun(1), ByteCnt(1), Data(2 per reg)
                 
                 // Extract Transaction ID
                 let trans_id_hi = buf[0];
                 let trans_id_lo = buf[1];
                 
                 // Construct response for 1 register (2 bytes data) value = 12345 (0x3039)
                 // Length = Unit(1) + Fun(1) + ByteCnt(1) + Data(2) = 5
                 let response = vec![
                     trans_id_hi, trans_id_lo, // Trans ID
                     0x00, 0x00, // Proto ID
                     0x00, 0x05, // Length
                     0x01, // Unit ID
                     0x04, // Function Code (Read Input Registers)
                     0x02, // Byte Count
                     0x30, 0x39  // Data (12345)
                 ];
                 
                 socket.write_all(&response).await.unwrap();
             }
        });

        let mapping = RegisterMapping {
            address: 100,
            count: 1,
            sensor_type: SensorType::Power,
            unit: "W".to_string(),
            scale: 1.0, 
        };

        let mut driver = ModbusDriver::new(
            "test-1".to_string(),
            addr,
            1,
            vec![mapping]
        );

        driver.connect().await.expect("Failed to connect");
        
        let readings = driver.read_all().await.expect("Failed to read");
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].value, 12345.0);
        assert_eq!(readings[0].unit, "W");
        
        driver.disconnect().await.unwrap();
    }
}
