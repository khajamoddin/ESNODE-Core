// ESNODE | Source Available BUSL-1.1 | Copyright (c) 2024 Estimatedstocks AB

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use crate::state::StatusSnapshot;

#[derive(Debug, Clone, Serialize)]
pub struct RiskAssessment {
    pub failure_probability: f64, // 0.0 to 1.0 (1.0 = imminent failure)
    pub risk_score: f64,          // 0 to 100
    pub factors: Vec<String>,
}

use serde::Serialize;

#[derive(Default)]
struct GpuHistory {
    ecc_corrected_aggr: VecDeque<(Instant, u64)>,
    ecc_uncorrected_aggr: VecDeque<(Instant, u64)>,
    throttle_events: VecDeque<(Instant, String)>,
    last_assessment: Option<RiskAssessment>,
}

pub struct FailureRiskPredictor {
    history: HashMap<String, GpuHistory>, // gpu_uuid -> History
    window_duration: Duration,
}

impl Default for FailureRiskPredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl FailureRiskPredictor {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
            window_duration: Duration::from_secs(3600), // 1 hour analysis window
        }
    }

    pub fn analyze(&mut self, snapshot: &StatusSnapshot) -> HashMap<String, RiskAssessment> {
        let now = Instant::now();
        let mut results = HashMap::new();

        for gpu in &snapshot.gpus {
                let uuid = gpu.uuid.clone().unwrap_or(gpu.gpu.clone());
                let history = self.history.entry(uuid.clone()).or_default();
                
                // Prune old history
                while let Some((ts, _)) = history.ecc_corrected_aggr.front() {
                    if now.duration_since(*ts) > self.window_duration {
                        history.ecc_corrected_aggr.pop_front();
                    } else {
                        break;
                    }
                }
                while let Some((ts, _)) = history.throttle_events.front() {
                     if now.duration_since(*ts) > self.window_duration {
                        history.throttle_events.pop_front();
                    } else {
                        break;
                    }
                }

                // 1. Update ECC History
                if let Some(h) = &gpu.health {
                    if let Some(c) = h.ecc_corrected_aggregate {
                         history.ecc_corrected_aggr.push_back((now, c));
                    }
                    if let Some(u) = h.ecc_uncorrected_aggregate {
                         history.ecc_uncorrected_aggr.push_back((now, u));
                    }
                    
                    // 2. Track Throttling
                    if !h.throttle_reasons.is_empty() {
                         for r in &h.throttle_reasons {
                             history.throttle_events.push_back((now, r.clone()));
                         }
                    }
                }

                // 3. Compute Risk Score
                let mut score: f64 = 0.0;
                let mut factors = Vec::new();
                let mut p_fail: f64 = 0.01; // Baseline 1% risk

                // Factor: Uncorrected ECC (Critical)
                if let Some(health) = &gpu.health {
                     if let Some(latest_u) = health.ecc_uncorrected_aggregate {
                         if latest_u > 0 {
                             score += 80.0;
                             p_fail += 0.5;
                             factors.push(format!("Has {} uncorrected ECC errors (Critical)", latest_u));
                         }
                     }
                }

                // Factor: Corrected ECC Rate
                if let (Some((_, first_c)), Some((_, last_c))) = (history.ecc_corrected_aggr.front(), history.ecc_corrected_aggr.back()) {
                    let delta = last_c.saturating_sub(*first_c);
                    if delta > 1000 {
                        score += 50.0;
                        p_fail += 0.2;
                        factors.push(format!("High rate of corrected ECC errors ({} in 1h)", delta));
                    } else if delta > 100 {
                        score += 20.0;
                        p_fail += 0.05;
                        factors.push(format!("Moderate corrected ECC errors ({} in 1h)", delta));
                    }
                }

                // Factor: Thermal Throttling Frequency
                let thermal_samples = history.throttle_events.iter().filter(|(_, r)| r.contains("thermal")).count();
                if thermal_samples > 10 { 
                     score += 30.0;
                     p_fail += 0.1;
                     factors.push(format!("Persistent thermal throttling detected ({} samples)", thermal_samples));
                }

                // Factor: Retired Pages
                if let Some(health) = &gpu.health {
                    if let Some(pages) = health.retired_pages {
                         if pages > 1 {
                             score += 40.0;
                             p_fail += 0.15;
                             factors.push(format!("Memory page retirement detected ({} pages)", pages));
                         }
                    }
                }

                let risk_score = score.min(100.0);
                let failure_probability = p_fail.min(1.0);

                let assessment = RiskAssessment {
                    failure_probability,
                    risk_score,
                    factors,
                };
                history.last_assessment = Some(assessment.clone());
                results.insert(uuid, assessment);
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{GpuStatus, GpuHealth, StatusSnapshot};

    #[test]
    fn test_predictor_high_risk() {
        let mut predictor = FailureRiskPredictor::new();
        
        let health = GpuHealth {
            ecc_uncorrected_aggregate: Some(5), // Critical!
            retired_pages: Some(2),
            ..Default::default()
        };
        let gpu = GpuStatus {
            uuid: Some("GPU-TEST-1".to_string()),
            health: Some(health),
            ..Default::default()
        };

        let snapshot = StatusSnapshot {
            gpus: vec![gpu],
            ..Default::default()
        };

        let analysis = predictor.analyze(&snapshot);
        let result = analysis.get("GPU-TEST-1").unwrap();

        assert!(result.risk_score >= 80.0);
        assert!(result.failure_probability >= 0.5);
        assert!(result.factors.iter().any(|f| f.contains("uncorrected ECC")));
        assert!(result.factors.iter().any(|f| f.contains("Memory page retirement")));
    }
}
