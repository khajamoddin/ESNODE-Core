use esnode_orchestrator::{Device, DeviceKind, LatencyClass, Orchestrator, OrchestratorConfig, Task};

#[test]
fn test_thermal_avoidance() {
    // Setup 2 devices: CPU1 (Cool), CPU2 (Hot)
    let dev1 = Device {
        id: "cpu1".to_string(),
        kind: DeviceKind::Cpu,
        peak_flops_tflops: 1.0,
        mem_gb: 32.0,
        power_watts_idle: 40.0,
        power_watts_max: 100.0,
        current_load: 0.1,
        last_seen: 0,
        // New fields
        temperature_celsius: Some(30.0), // Cool
        real_power_watts: Some(45.0),
        assigned_tasks: vec![],
    };

    let dev2 = Device {
        id: "cpu2".to_string(),
        kind: DeviceKind::Cpu,
        peak_flops_tflops: 1.0,
        mem_gb: 32.0,
        power_watts_idle: 40.0,
        power_watts_max: 100.0,
        current_load: 0.1,
        last_seen: 0,
        // New fields
        temperature_celsius: Some(95.0), // Hot!
        real_power_watts: Some(95.0),
        assigned_tasks: vec![],
    };

    let config = OrchestratorConfig {
        enable_turbo_mode: false,
        enable_zombie_reaper: false,
        enable_thermal_management: true, 
        ..OrchestratorConfig::default()
    };

    // Initialize Orchestrator
    let orch = Orchestrator::new(vec![dev1, dev2], config);

    let task = Task {
        id: "hot_task".to_string(),
        est_flops: 1e11,
        est_bytes: 1e8,
        latency_class: LatencyClass::Medium,
        preferred_kinds: None,
    };

    // Should pick cpu1 because cpu2 is hot
    let chosen = orch.pick_device_for_task(&task).expect("Should pick a device");
    assert_eq!(chosen, "cpu1", "Should have picked cpu1 (30C) over cpu2 (95C)");
}
