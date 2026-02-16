use crate::collectors::Collector;
use crate::state::StatusState;
use crate::MetricsRegistry;
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use tracing::{debug, warn};

/// PUE (Power Usage Effectiveness) Calculator
/// 
/// Calculates real-time data center efficiency:
/// PUE = Total Facility Power / IT Equipment Power
/// 
/// Ideal PUE = 1.0 (all power goes to IT)
/// Typical PUE = 1.5-2.0 (facilities, cooling, lighting)
pub struct PueCalculator {
    _status: StatusState,
    /// Cached IT equipment power by source (watts)
    it_power_sources: Arc<RwLock<HashMap<String, f64>>>,
    /// Cached total facility power by source (watts)
    facility_power_sources: Arc<RwLock<HashMap<String, f64>>>,
}

impl PueCalculator {
    pub fn new(status: StatusState) -> Self {
        Self {
            _status: status,
            it_power_sources: Arc::new(RwLock::new(HashMap::new())),
            facility_power_sources: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Calculate PUE from current power readings
    fn calculate_pue(&self) -> f64 {
        let it = self.total_it_power();
        let facility = self.total_facility_power();

        if it <= 0.0 {
            debug!("IT power is zero or negative, cannot calculate PUE");
            return 0.0;
        }

        let pue = facility / it;

        // Sanity check: PUE should be >= 1.0
        if pue < 1.0 {
            warn!("PUE < 1.0 detected ({:.3}). Facility power may be under-reported.", pue);
        }

        // Cap unrealistic values
        if pue > 10.0 {
            warn!("PUE > 10.0 detected ({:.3}). This is highly unusual.", pue);
            return 10.0;
        }

        pue
    }

    pub fn total_it_power(&self) -> f64 {
        self.it_power_sources.read().values().sum()
    }

    pub fn total_facility_power(&self) -> f64 {
        self.facility_power_sources.read().values().sum()
    }

    /// Update IT equipment power from a specific source (e.g., "gpu-0", "cpu-package")
    pub fn update_it_power(&self, source: &str, watts: f64) {
        let mut sources = self.it_power_sources.write();
        sources.insert(source.to_string(), watts);
        debug!("Updated IT power from {}: {:.2}W (Total: {:.2}W)", source, watts, sources.values().sum::<f64>());
    }

    /// Update facility power from a specific source (e.g., "pdu-1", "ups-a")
    pub fn update_facility_power(&self, source: &str, watts: f64) {
        let mut sources = self.facility_power_sources.write();
        sources.insert(source.to_string(), watts);
        debug!("Updated facility power from {}: {:.2}W (Total: {:.2}W)", source, watts, sources.values().sum::<f64>());
    }

    /// Calculate efficiency (inverse of PUE, expressed as percentage)
    /// Efficiency = (IT Power / Total Power) * 100
    fn calculate_efficiency(&self) -> f64 {
        let it = self.total_it_power();
        let facility = self.total_facility_power();

        if facility <= 0.0 {
            return 0.0;
        }

        (it / facility) * 100.0
    }

    /// Calculate overhead power (cooling, lighting, etc.)
    fn calculate_overhead(&self) -> f64 {
        let it = self.total_it_power();
        let facility = self.total_facility_power();

        (facility - it).max(0.0)
    }
}

#[async_trait]
impl Collector for PueCalculator {
    fn name(&self) -> &'static str {
        "pue"
    }

    async fn collect(&mut self, metrics: &MetricsRegistry) -> anyhow::Result<()> {
        self.collect_internal(metrics).await
    }
}

impl PueCalculator {
    pub async fn collect_internal(&self, metrics: &MetricsRegistry) -> anyhow::Result<()> {
        let pue = self.calculate_pue();
        let efficiency = self.calculate_efficiency();
        let overhead = self.calculate_overhead();
        
        let it_power = self.total_it_power();
        let facility_power = self.total_facility_power();

        // Export PUE metric
        metrics.pue_ratio.set(pue);
        
        // Export supporting metrics
        metrics.pue_it_power_watts.set(it_power);
        metrics.pue_facility_power_watts.set(facility_power);
        metrics.pue_efficiency_percent.set(efficiency);
        metrics.pue_overhead_watts.set(overhead);

        debug!(
            "PUE calculated: {:.3} (IT: {:.0}W, Facility: {:.0}W, Efficiency: {:.1}%)",
            pue, it_power, facility_power, efficiency
        );

        Ok(())
    }
}

/// Helper to aggregate power from multiple sources
#[derive(Clone)]
pub struct PowerAggregator {
    pue_calculator: Arc<PueCalculator>,
}

/// Wrapper to allow sharing PueCalculator between collectors and the aggregator
pub struct PueCollectorWrapper {
    pub calculator: Arc<PueCalculator>,
}

#[async_trait]
impl Collector for PueCollectorWrapper {
    fn name(&self) -> &'static str {
        "pue"
    }

    async fn collect(&mut self, metrics: &MetricsRegistry) -> anyhow::Result<()> {
        self.calculator.collect_internal(metrics).await
    }
}

impl PowerAggregator {
    pub fn new(pue_calculator: Arc<PueCalculator>) -> Self {
        Self { pue_calculator }
    }

    /// Update IT power from node telemetry
    /// Call this from GPU/CPU collectors
    pub fn report_it_power(&self, source: &str, watts: f64) {
        self.pue_calculator.update_it_power(source, watts);
    }

    /// Update facility power from IoT sensors
    /// Call this from MQTT/SNMP collectors
    pub fn report_facility_power(&self, source: &str, watts: f64) {
        self.pue_calculator.update_facility_power(source, watts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn test_pue_calculation() {
        let healthy = Arc::new(AtomicBool::new(true));
        let status = StatusState::new(healthy);
        let calc = PueCalculator::new(status);

        // Ideal scenario: IT = 1000W, Facility = 1000W → PUE = 1.0
        calc.update_it_power("server-1", 1000.0);
        calc.update_facility_power("pdu-1", 1000.0);
        assert_eq!(calc.calculate_pue(), 1.0);

        // Multiple sources
        calc.update_it_power("server-2", 500.0);
        calc.update_facility_power("pdu-2", 1000.0); // Total IT: 1500, Total Facility: 2000
        assert_eq!(calc.calculate_pue(), 2000.0 / 1500.0);
    }
}
