// ESNODE | Source Available BUSL-1.1 | Copyright (c) 2024 Estimatedstocks AB
use async_trait::async_trait;

use crate::metrics::MetricsRegistry;

pub mod app;
#[cfg(feature = "ebpf")]
pub mod ebpf;
pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod numa;
pub mod power;
pub mod protocol_runner;
pub mod pue;

#[async_trait]
pub trait Collector: Send {
    fn name(&self) -> &'static str;
    async fn collect(&mut self, metrics: &MetricsRegistry) -> anyhow::Result<()>;
}
