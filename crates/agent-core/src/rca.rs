// ESNODE | Source Available BUSL-1.1 | Copyright (c) 2024 Estimatedstocks AB

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use crate::state::StatusSnapshot;

#[derive(Debug, Clone, PartialEq)]
pub enum RootCause {
    NetworkLatency,
    ThermalThrottling,
    PowerThrottling,
    KubernetesEvents,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct RcaEvent {
    pub timestamp: Instant,
    pub cause: RootCause,
    pub description: String,
    pub confidence: f64, // 0.0 to 1.0
}

pub struct AnalysisWindow {
    capacity: usize,
    samples: VecDeque<(Instant, StatusSnapshot)>,
}

impl AnalysisWindow {
    pub fn new(duration: Duration, scrape_interval: Duration) -> Self {
        let capacity = (duration.as_secs_f64() / scrape_interval.as_secs_f64()).ceil() as usize;
        Self {
            capacity,
            samples: VecDeque::with_capacity(capacity),
        }
    }

    pub fn add(&mut self, snapshot: StatusSnapshot) {
        if self.samples.len() >= self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back((Instant::now(), snapshot));
    }

    pub fn samples(&self) -> &VecDeque<(Instant, StatusSnapshot)> {
        &self.samples
    }
}

pub struct RcaEngine {
    window: AnalysisWindow,
}

impl RcaEngine {
    pub fn new(window_duration: Duration, scrape_interval: Duration) -> Self {
        Self {
            window: AnalysisWindow::new(window_duration, scrape_interval),
        }
    }

    pub fn add_snapshot(&mut self, snapshot: StatusSnapshot) {
        self.window.add(snapshot);
    }

    pub fn analyze(&self) -> Vec<RcaEvent> {
        let mut events = Vec::new();
        let samples = self.window.samples();

        if samples.len() < 2 {
            return events;
        }

        // Get latest and previous
        let (_latest_ts, latest) = samples.back().unwrap();
        let (_prev_ts, prev) = samples.get(samples.len() - 2).unwrap();

        // Check for sudden GPU utilization drop
        for (idx, gpu) in latest.gpus.iter().enumerate() {
            if let Some(prev_gpu) = prev.gpus.get(idx) {
                let curr_util = gpu.util_percent.unwrap_or(0.0);
                let prev_util = prev_gpu.util_percent.unwrap_or(0.0);

                // If utilization dropped significantly (e.g., > 20% drop)
                if prev_util > 50.0 && curr_util < (prev_util - 20.0) {
                    // Utilization Dip Detected. Look for causes.
                    
                    // Check 1: Network Latency / Degradation
                    if self.check_network_cause() {
                        events.push(RcaEvent {
                            timestamp: Instant::now(),
                            cause: RootCause::NetworkLatency,
                            description: format!("GPU-{} utilization dropped from {:.1}% to {:.1}% coincident with network degradation", 
                                idx, prev_util, curr_util),
                            confidence: 0.8,
                        });
                        continue;
                    }

                    // Check 2: Thermal Throttling
                    if gpu.thermal_throttle {
                         events.push(RcaEvent {
                            timestamp: Instant::now(),
                            cause: RootCause::ThermalThrottling,
                            description: format!("GPU-{} utilization dropped due to thermal throttling", idx),
                            confidence: 1.0,
                        });
                        continue;
                    }

                    // Check 3: Kubernetes Pod Events
                    if latest.k8s_events_detected {
                        events.push(RcaEvent {
                            timestamp: Instant::now(),
                            cause: RootCause::KubernetesEvents,
                            description: format!("GPU-{} utilization drop correlates with Kubernetes pod events (evictions/rescheduling)", idx),
                            confidence: 0.9,
                        });
                        continue;
                    }
                }
            }
        }

        events
    }

    fn check_network_cause(&self) -> bool {
        let samples = self.window.samples();
        for (_, snapshot) in samples.iter().rev().take(3) { 
            if snapshot.network_degraded {
                 return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StatusState;
    use std::sync::{Arc, atomic::AtomicBool};

    #[test]
    fn test_window_logic() {
        let mut window = AnalysisWindow::new(Duration::from_secs(10), Duration::from_secs(1));
        assert_eq!(window.samples.capacity(), 10);
        
        let healthy = Arc::new(AtomicBool::new(true));
        let status = StatusState::new(healthy);
        
        for _ in 0..15 {
            window.add(status.snapshot());
        }
        
        assert_eq!(window.samples.len(), 10);
    }
}
