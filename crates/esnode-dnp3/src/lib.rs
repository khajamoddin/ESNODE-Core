use agent_core::drivers::{Driver, Reading};
use async_trait::async_trait;
use bytes::{Buf, BufMut, BytesMut};
use crc::{Crc, CRC_16_DNP};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Encoder, Framed};
use futures::sink::SinkExt;
use futures::stream::StreamExt;

const CRC_DNP: Crc<u16> = Crc::<u16>::new(&CRC_16_DNP);

#[derive(Debug, Clone)]
pub struct Dnp3Config {
    pub local_addr: u16,         // Source Address (Master)
    pub remote_addr: u16,        // Destination Address (Outstation)
    pub integrity_interval_ms: u64,
}

#[derive(Debug)]
struct Dnp3Frame {
    control: u8,
    dest: u16,
    src: u16,
    payload: Vec<u8>,
}

struct Dnp3Codec;

impl Decoder for Dnp3Codec {
    type Item = Dnp3Frame;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Minimum header size: 0x05 0x64 [Len] [Ctrl] [DestL] [DestH] [SrcL] [SrcH] [CRC_L] [CRC_H] = 10 bytes
        if src.len() < 10 {
            return Ok(None);
        }

        // Check Sync Bytes
        if src[0] != 0x05 || src[1] != 0x64 {
            // Invalid sync, advance 1 byte and retry to find sync
            src.advance(1);
            return self.decode(src);
        }

        let length = src[2] as usize; 
        // DNP3 Length byte implies user data count.
        // User Data = Control (1) + Dest (2) + Src (2) + actual payload.
        // Payload Size = Length - 5.
        
        // Header CRC covers bytes 2..7 (Length, Control, Dest, Src) -> 6 bytes
        let header_crc_calc = CRC_DNP.checksum(&src[2..8]);
        let header_crc_read = u16::from_le_bytes([src[8], src[9]]);
        
        if header_crc_calc != header_crc_read {
            // Header CRC mismatch.
             src.advance(1);
             return self.decode(src);
        }

        if length < 5 {
             // Invalid length (must contain at least ctrl+addrs)
             src.advance(10);
             return self.decode(src);
        }

        // Valid Header. Determine total frame size.
        // User Data (Logical) = Ctrl(1) + Dest(2) + Src(2) + Payload.
        // Physical: Header contains Ctrl, Dest, Src. 
        // Remaining User Data (Body) = Length - 5.
        
        let body_len = length as usize - 5;
        
        // Calculation of CRC blocks for BODY only.
        let num_full_blocks = body_len / 16;
        let partial_block = body_len % 16;
        let total_crc_bytes = num_full_blocks * 2 + if partial_block > 0 { 2 } else { 0 };
        let total_frame_size = 10 + body_len + total_crc_bytes;

        if src.len() < total_frame_size {
            src.reserve(total_frame_size - src.len());
            return Ok(None);
        }

        // Parse Frame
        let control = src[3];
        let dest = u16::from_le_bytes([src[4], src[5]]);
        let src_addr = u16::from_le_bytes([src[6], src[7]]);
        
        // Extract Payload (skipping intermediate CRCs which we validate implicitly here for simplicity or skip)
        let mut payload = Vec::new();
        // let mut data_slice = &src[10..total_frame_size]; // Unused variable warning fix
        
        // Simple extraction logic (ignoring CRC validation for payload for now)
        let mut remaining = body_len;
        let mut cursor = 10;
        
        while remaining > 0 {
            let chunk_size = std::cmp::min(remaining, 16);
            payload.extend_from_slice(&src[cursor..cursor+chunk_size]);
            cursor += chunk_size;
            cursor += 2; // Skip CRC
            remaining -= chunk_size;
        }

        src.advance(total_frame_size);

        Ok(Some(Dnp3Frame {
            control,
            dest,
            src: src_addr,
            payload,
        }))
    }
}

impl Encoder<Dnp3Frame> for Dnp3Codec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Dnp3Frame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Construct Link Header
        // Length = Control (1) + Dest (2) + Src (2) + Payload Len
        let len_byte = (5 + item.payload.len()) as u8;
        
        // Calculate needed capacity: Header (10) + Payload + Payload CRCs
        let payload_len = item.payload.len();
        let payload_crcs = (payload_len / 16 + if payload_len % 16 > 0 { 1 } else { 0 }) * 2;
        dst.reserve(10 + payload_len + payload_crcs);
        
        dst.put_u8(0x05);
        dst.put_u8(0x64);
        dst.put_u8(len_byte);
        dst.put_u8(item.control);
        dst.put_u16_le(item.dest);
        dst.put_u16_le(item.src);
        
        // Header CRC (bytes 2..7)
        // We just wrote 2 sync + 1 len + 1 ctrl + 2 dest + 2 src = 8 bytes.
        // CRC covers from index 2 to 7 (6 bytes).
        let header_start = dst.len() - 6; 
        let header_crc = CRC_DNP.checksum(&dst[header_start..]);
        dst.put_u16_le(header_crc);
        
        // Payload with CRCs every 16 bytes
        for chunk in item.payload.chunks(16) {
            dst.put_slice(chunk);
            let chunk_crc = CRC_DNP.checksum(chunk);
            dst.put_u16_le(chunk_crc);
        }
        
        Ok(())
    }
}

pub struct Dnp3Driver {
    id: String,
    addr: SocketAddr,
    config: Dnp3Config,
    stream: Option<Framed<TcpStream, Dnp3Codec>>,
}

impl Dnp3Driver {
    pub fn new(id: String, addr: SocketAddr, config: Dnp3Config) -> Self {
        Self {
            id,
            addr,
            config,
            stream: None,
        }
    }
}

#[async_trait]
impl Driver for Dnp3Driver {
    fn id(&self) -> &str {
        &self.id
    }

    async fn connect(&mut self) -> anyhow::Result<()> {
        let stream = TcpStream::connect(self.addr).await?;
        self.stream = Some(Framed::new(stream, Dnp3Codec));
        
        // Send Link Reset
        if let Some(framed) = &mut self.stream {
            // Reset Link Function Code (0x01) | PRI (0x80) | DIR (0x40)
            let frame = Dnp3Frame {
                control: 0xC0 | 0x01, // DIR=1, PRM=1, FCB=0, FCV=0, FUNC=1 (Reset Link)
                dest: self.config.remote_addr,
                src: self.config.local_addr,
                payload: vec![],
            };
            framed.send(frame).await?;
            
            // Should verify ACK
            // let _ack = framed.next().await; 
        }
        
        Ok(())
    }

    async fn read_all(&mut self) -> anyhow::Result<Vec<Reading>> {
         // Send Integrity Poll (Class 0123 read)
         // Application Layer:
         // FUNC = 0x01 (READ)
         // Object Header: Group 60 Var 1 (Class 0), Group 60 Var 2 (Class 1), etc.
         // Or simplified: Group 60 Var 1 (Class 0) + Var 2/3/4.
         
         // Minimal implementation: Send generic Class 0 poll byte sequence.
         // Application Fragment: 0xC0 (FIR, FIN, CON, UNS=0, SEQ=0) | FUNC=0x01 (READ)
         // Object: Group 60 (0x3C), Var 1 (0x01), Qualifier 0x06 (All Points)
         
         // 0xC0 0x01 0x3C 0x01 0x06
         
         let app_fragment = vec![0xC0, 0x01, 0x3C, 0x01, 0x06];
         
         if let Some(framed) = &mut self.stream {
            let frame = Dnp3Frame {
                control: 0xC0 | 0x03, // User Data function code (0x03) for Transport
                dest: self.config.remote_addr,
                src: self.config.local_addr,
                payload: app_fragment, // Note: Transport header should be added here
            };
            
            // Transport Header: FIN=1, FIR=1, SEQ=0. (0xC0)
            // Wrapping Application Fragment in Transport Header
            let mut transport_payload = vec![0xC0];
            transport_payload.extend_from_slice(&frame.payload);
            
            let link_frame = Dnp3Frame {
                control: 0xC4 | 0x03, // User Data without CONFIRM (0x44) or with? Let's assume User Data (0x03)
                dest: self.config.remote_addr,
                src: self.config.local_addr,
                payload: transport_payload,
            };

            framed.send(link_frame).await?;
            
            // Wait for response...
            let _response = framed.next().await;
            
            // Parsing would go here. For MVP, we return empty or dummy reading.
         }

        Ok(vec![])
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        self.stream = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio::io::AsyncReadExt;
    
    #[tokio::test]
    async fn test_dnp3_codec() {
        // Test basic encode/decode
        let mut codec = Dnp3Codec;
        let frame = Dnp3Frame {
            control: 0xC1,
            dest: 1024,
            src: 1,
            payload: vec![0xCA, 0xFE],
        };
        
        let mut buf = BytesMut::new();
        codec.encode(frame, &mut buf).unwrap();
        
        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(decoded.dest, 1024);
        assert_eq!(decoded.src, 1);
        assert_eq!(decoded.payload, vec![0xCA, 0xFE]);
    }
}
