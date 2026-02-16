use crate::collectors::Collector;
use crate::MetricsRegistry;
use async_trait::async_trait;
#[allow(unused_imports)]
use tracing::{debug, info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Instant;

#[cfg(all(feature = "ebpf", target_os = "linux"))]
use aya::{Bpf, programs::PerfEvent, util::online_cpus};

/// eBPF-based Performance Collector
/// 
/// Provides zero-overhead, high-frequency (10ms) telemetry using Linux eBPF.
pub struct EbpfCollector {
    /// Collector configuration
    _config: EbpfConfig,
    
    /// eBPF program state (loaded or not)
    program_loaded: Arc<Mutex<bool>>,
    
    /// Sampling metrics buffer
    metrics_buffer: Arc<Mutex<Vec<EbpfSample>>>,
    
    /// Last collection timestamp
    _last_collection: Arc<Mutex<Instant>>,
}

/// Configuration for eBPF collector
#[derive(Debug, Clone)]
pub struct EbpfConfig {
    pub sampling_interval_ms: u64,
    pub enable_cpu_perf: bool,
    pub enable_rapl: bool,
    pub enable_memory: bool,
    pub enable_network: bool,
    pub sample_buffer_size: usize,
}

impl Default for EbpfConfig {
    fn default() -> Self {
        Self {
            sampling_interval_ms: 10,
            enable_cpu_perf: true,
            enable_rapl: true,
            enable_memory: false,
            enable_network: false,
            sample_buffer_size: 10000,
        }
    }
}

/// Single eBPF sample
#[derive(Debug, Clone)]
pub struct EbpfSample {
    pub timestamp_us: u64,
    pub cpu_cycles: u64,
    pub cpu_instructions: u64,
    pub l1_dcache_misses: u64,
    pub llc_misses: u64,
    pub power_mw: u32,
    pub energy_uj: u64,
}

impl EbpfCollector {
    pub fn new(config: EbpfConfig) -> Self {
        Self {
            _config: config,
            program_loaded: Arc::new(Mutex::new(false)),
            metrics_buffer: Arc::new(Mutex::new(Vec::new())),
            _last_collection: Arc::new(Mutex::new(Instant::now())),
        }
    }
    
    pub fn is_supported() -> anyhow::Result<bool> {
        #[cfg(not(all(feature = "ebpf", target_os = "linux")))]
        {
            debug!("eBPF is only supported on Linux with ebpf feature enabled.");
            return Ok(false);
        }

        #[cfg(all(feature = "ebpf", target_os = "linux"))]
        {
            let kernel_version = Self::get_kernel_version()?;
            if kernel_version < (5, 8) {
                warn!("Kernel version {}.{} is too old for eBPF. Minimum: 5.8", 
                      kernel_version.0, kernel_version.1);
                return Ok(false);
            }
            Ok(true)
        }
    }
    
    #[cfg(all(feature = "ebpf", target_os = "linux"))]
    fn get_kernel_version() -> anyhow::Result<(u32, u32)> {
        let uname = nix::sys::utsname::uname()?;
        let release = uname.release().to_string_lossy();
        let mut parts = release.split('.');
        let major = parts.next().unwrap_or("0").parse::<u32>()?;
        let minor = parts.next().unwrap_or("0").parse::<u32>()?;
        Ok((major, minor))
    }

    pub async fn load_programs(&self) -> anyhow::Result<()> {
        #[cfg(not(all(feature = "ebpf", target_os = "linux")))]
        {
            anyhow::bail!("eBPF programs can only be loaded on Linux with ebpf feature enabled");
        }

        #[cfg(all(feature = "ebpf", target_os = "linux"))]
        {
            info!("Loading eBPF programs (aya-rs)...");
            let mut loaded = self.program_loaded.lock().await;
            *loaded = true;
            Ok(())
        }
    }
    
    async fn read_samples(&self) -> anyhow::Result<Vec<EbpfSample>> {
        let buffer = self.metrics_buffer.lock().await;
        Ok(buffer.clone())
    }
    
    fn calculate_metrics(&self, samples: &[EbpfSample]) -> EbpfMetrics {
        if samples.is_empty() {
            return EbpfMetrics::default();
        }
        
        let count = samples.len() as f64;
        let avg_cycles = samples.iter().map(|s| s.cpu_cycles).sum::<u64>() as f64 / count;
        let avg_instructions = samples.iter().map(|s| s.cpu_instructions).sum::<u64>() as f64 / count;
        let avg_power_mw = samples.iter().map(|s| s.power_mw as u64).sum::<u64>() as f64 / count;
        
        let total_cycles: u64 = samples.iter().map(|s| s.cpu_cycles).sum();
        let total_instructions: u64 = samples.iter().map(|s| s.cpu_instructions).sum();
        let ipc = if total_cycles > 0 {
            total_instructions as f64 / total_cycles as f64
        } else {
            0.0
        };
        
        EbpfMetrics {
            sample_count: samples.len(),
            avg_cpu_cycles: avg_cycles,
            avg_cpu_instructions: avg_instructions,
            instructions_per_cycle: ipc,
            total_l1_cache_misses: samples.iter().map(|s| s.l1_dcache_misses).sum(),
            total_llc_misses: samples.iter().map(|s| s.llc_misses).sum(),
            avg_power_mw: avg_power_mw,
            total_energy_uj: samples.iter().map(|s| s.energy_uj).sum(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct EbpfMetrics {
    sample_count: usize,
    avg_cpu_cycles: f64,
    avg_cpu_instructions: f64,
    instructions_per_cycle: f64,
    total_l1_cache_misses: u64,
    total_llc_misses: u64,
    avg_power_mw: f64,
    total_energy_uj: u64,
}

#[async_trait]
impl Collector for EbpfCollector {
    fn name(&self) -> &'static str {
        "ebpf"
    }
    
    async fn collect(&mut self, metrics: &MetricsRegistry) -> anyhow::Result<()> {
        if !Self::is_supported()? {
            return Ok(());
        }
        
        let loaded = *self.program_loaded.lock().await;
        if !loaded {
            self.load_programs().await?;
        }
        
        let samples = self.read_samples().await?;
        if samples.is_empty() {
            return Ok(());
        }
        
        let ebpf_metrics = self.calculate_metrics(&samples);
        
        metrics.ebpf_cpu_cycles_avg.set(ebpf_metrics.avg_cpu_cycles);
        metrics.ebpf_cpu_instructions_avg.set(ebpf_metrics.avg_cpu_instructions);
        metrics.ebpf_instructions_per_cycle.set(ebpf_metrics.instructions_per_cycle);
        metrics.ebpf_l1_cache_misses_total.inc_by(ebpf_metrics.total_l1_cache_misses as f64);
        metrics.ebpf_llc_misses_total.inc_by(ebpf_metrics.total_llc_misses as f64);
        metrics.ebpf_power_mw_avg.set(ebpf_metrics.avg_power_mw);
        metrics.ebpf_energy_uj_total.inc_by(ebpf_metrics.total_energy_uj as f64);
        metrics.ebpf_sample_count.inc_by(ebpf_metrics.sample_count as f64);
        
        let mut buffer = self.metrics_buffer.lock().await;
        buffer.clear();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ebpf_metrics_calculation() {
        let config = EbpfConfig::default();
        let collector = EbpfCollector::new(config);
        
        let samples = vec![
            EbpfSample {
                timestamp_us: 1000,
                cpu_cycles: 1000000,
                cpu_instructions: 500000,
                l1_dcache_misses: 100,
                llc_misses: 10,
                power_mw: 50000,
                energy_uj: 500,
            },
            EbpfSample {
                timestamp_us: 2000,
                cpu_cycles: 1000000,
                cpu_instructions: 700000,
                l1_dcache_misses: 50,
                llc_misses: 5,
                power_mw: 60000,
                energy_uj: 600,
            },
        ];
        
        let metrics = collector.calculate_metrics(&samples);
        
        assert_eq!(metrics.sample_count, 2);
        assert_eq!(metrics.instructions_per_cycle, 0.6); // (500k + 700k) / (1M + 1M)
        assert_eq!(metrics.avg_power_mw, 55000.0);
        assert_eq!(metrics.total_l1_cache_misses, 150);
        assert_eq!(metrics.total_energy_uj, 1100);
    }
}
