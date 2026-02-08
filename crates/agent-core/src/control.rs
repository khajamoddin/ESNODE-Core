// ESNODE | Source Available BUSL-1.1 | Copyright (c) 2024 Estimatedstocks AB

use crate::policy::{ActionType, PolicyAction};
use anyhow::{anyhow, Result};
#[cfg(feature = "gpu")]
use nvml_wrapper::Nvml;
use tracing::{info, warn};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct Enforcer {
    #[cfg(feature = "gpu")]
    nvml: Option<Nvml>,
}

impl Default for Enforcer {
    fn default() -> Self {
        Self::new()
    }
}

impl Enforcer {
    pub fn new() -> Self {
        #[cfg(feature = "gpu")]
        let nvml = match Nvml::init() {
            Ok(n) => Some(n),
            Err(e) => {
                warn!("Failed to initialize NVML for enforcement: {}", e);
                None
            }
        };

        Self {
            #[cfg(feature = "gpu")]
            nvml,
        }
    }

    pub fn apply_action(&self, target_resource: &str, action: &PolicyAction) -> Result<String> {
        match action.action_type {
            ActionType::ThrottlePower => self.apply_throttle_power(target_resource, action),
            ActionType::LockClock => self.apply_lock_clock(target_resource, action),
            ActionType::Alert => self.apply_alert(target_resource, action),
            ActionType::KillProcess => self.apply_kill_process(target_resource, action),
            ActionType::MigratePod => self.apply_migrate_pod(target_resource, action),
        }
    }

    fn apply_throttle_power(&self, target: &str, action: &PolicyAction) -> Result<String> {
        #[cfg(feature = "gpu")]
        {
            let Some(nvml) = &self.nvml else {
                return Err(anyhow!("NVML not available, cannot throttle power"));
            };

            // Target expected format: "GPU-<UUID>" or "GPU-<INDEX>"
            let device = if let Some(uuid) = target.strip_prefix("GPU-") {
                if let Ok(idx) = uuid.parse::<u32>() {
                    nvml.device_by_index(idx)
                } else {
                    nvml.device_by_uuid(uuid)
                }
            } else {
                // Fallback, treat entire string as UUID or Index if possible
                 if let Ok(idx) = target.parse::<u32>() {
                    nvml.device_by_index(idx)
                } else {
                    nvml.device_by_uuid(target)
                }
            };
            
            let mut device = device.map_err(|e| anyhow!("Failed to find device {}: {}", target, e))?;

            // Parameters: "limit_watts" or "limit"
            let limit_val = action.parameters.get("limit_watts")
                .or_else(|| action.parameters.get("limit"))
                .ok_or_else(|| anyhow!("Missing 'limit_watts' parameter for throttle_power"))?;

            let limit_watts = limit_val.as_f64()
                .ok_or_else(|| anyhow!("'limit_watts' must be a number"))?;

            let limit_microwatts = (limit_watts * 1000.0) as u32;

            // Check constraints
            let constraints = device.power_management_limit_constraints()
                .map_err(|e| anyhow!("Failed to get power constraints: {}", e))?;
            
            if limit_microwatts < constraints.min_limit || limit_microwatts > constraints.max_limit {
                 return Err(anyhow!(
                    "Requested power limit {:.1}W is out of range ({:.1}W - {:.1}W)", 
                    limit_watts, 
                    constraints.min_limit as f64 / 1000.0, 
                    constraints.max_limit as f64 / 1000.0
                ));
            }

            device.set_power_management_limit(limit_microwatts)
                .map_err(|e| anyhow!("Failed to set power limit: {}", e))?;

            let msg = format!("Throttled {} to {:.1}W", target, limit_watts);
            info!("{}", msg);
            Ok(msg)
        }
        #[cfg(not(feature = "gpu"))]
        {
            Err(anyhow!("GPU feature not enabled"))
        }
    }

    fn apply_lock_clock(&self, _target: &str, _action: &PolicyAction) -> Result<String> {
        // Placeholder for clock locking implementation
        // This requires `set_gpu_locked_clocks`
        Ok("Clock locking simulated (not yet fully implemented)".to_string())
    }

    fn apply_alert(&self, target: &str, action: &PolicyAction) -> Result<String> {
        let msg = action.parameters.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Policy violation detected");
        
        // In a real system, this might send a webhook, Slack message, etc.
        // For now, we just log it as an "applied action".
        let out = format!("ALERT on {}: {}", target, msg);
        warn!("{}", out);
        Ok(out)
    }

    fn apply_kill_process(&self, _target: &str, _action: &PolicyAction) -> Result<String> {
        // Implementation would need to find processes using the GPU (nvmlDeviceGetComputeRunningProcesses)
        // and kill them. Dangerous!
        Ok("Kill process simulated (safety lock active)".to_string())
    }

    fn apply_migrate_pod(&self, _target: &str, _action: &PolicyAction) -> Result<String> {
        // Would interface with K8s API to drain/cordon node or delete pod.
        Ok("Pod migration simulated (K8s integration pending)".to_string())
    }
}

pub struct FlapDampener {
    last_actions: HashMap<(String, String), Instant>, // (policy, target) -> timestamp
    dampening_interval: Duration,
}

impl FlapDampener {
    pub fn new(dampening_interval: Duration) -> Self {
        Self {
            last_actions: HashMap::new(),
            dampening_interval,
        }
    }

    pub fn can_apply(&self, policy: &str, target: &str) -> bool {
        if let Some(last) = self.last_actions.get(&(policy.to_string(), target.to_string())) {
            if last.elapsed() < self.dampening_interval {
                return false;
            }
        }
        true
    }

    pub fn record_action(&mut self, policy: &str, target: &str) {
        self.last_actions.insert((policy.to_string(), target.to_string()), Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flap_dampener() {
        let mut dampener = FlapDampener::new(Duration::from_millis(100));
        let policy = "test_policy";
        let target = "test_target";

        // First check should pass (no record)
        assert!(dampener.can_apply(policy, target));

        // Record action
        dampener.record_action(policy, target);

        // Immediate check should fail (dampened)
        assert!(!dampener.can_apply(policy, target));

        // Different target should pass
        assert!(dampener.can_apply(policy, "other_target"));

        // Wait for interval
        std::thread::sleep(Duration::from_millis(150));

        // Should pass again
        assert!(dampener.can_apply(policy, target));
    }
}
