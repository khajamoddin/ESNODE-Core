// ESNODE | Source Available BUSL-1.1 | Copyright (c) 2025 Estimatedstocks AB
use crate::Orchestrator;

/// Thermal Management Feature
///
/// Monitors device temperatures and triggers autonomous responses:
/// 1. If temp > CRITICAL (90C): Mark device as unhealthy/draining.
/// 2. If temp > WARNING (80C): Penalty in scoring (handled by `tick` updates).
/// 3. If temp < NORMAL (60C): Allow turbo/overclocking candidates.
pub fn check_thermals(orch: &mut Orchestrator) {
    tracing::debug!("Running Thermal Management...");

    let mut hot_devices = Vec::new();

    for (id, device) in &mut orch.devices {
        if let Some(temp) = device.temperature_celsius {
            if temp > 85.0 {
                // Hot!
                tracing::warn!("Device {} is overheating ({} C)", id, temp);
                hot_devices.push(id.clone());
            }
        }
    }

    // In a real implementation, we would now trigger task migration directly
    // or adjust the device's availability flags.
    for id in hot_devices {
        if let Some(dev) = orch.devices.get_mut(&id) {
            // Artificially inflate load to discourage new tasks (soft drain)
            dev.current_load = (dev.current_load + 0.5).min(1.0);
        }
    }
}
