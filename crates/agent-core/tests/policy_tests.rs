#[cfg(test)]
mod tests {
    use agent_core::policy::{EfficiencyProfile, PlanStatus};
    use agent_core::state::{GpuStatus, StatusSnapshot};

    fn mock_snapshot() -> StatusSnapshot {
        let gpu = GpuStatus {
            uuid: Some("GPU-123".to_string()),
            gpu: "NVIDIA H100".to_string(),
            temperature_celsius: Some(85.0), // Should violate > 80
            util_percent: Some(2.0),         // Should violate < 5
            ..Default::default()
        };

        StatusSnapshot {
            gpus: vec![gpu],
            ..Default::default()
        }
    }

    #[test]
    fn test_plan_thermal_violation() {
        let yaml = r#"
        apiVersion: v1
        kind: EfficiencyProfile
        metadata:
          name: "test-profile"
          version: "1.0.0"
        selectors: {}
        policies:
          - name: "thermal-safety"
            target: gpu_temp_celsius
            condition: "> 80"
            severity: critical
            action:
              type: throttle_power
              parameters: { min: 300 }
        "#;

        let profile: EfficiencyProfile = serde_yaml::from_str(yaml).unwrap();
        let status = mock_snapshot();
        let result = profile.plan(&status);

        assert_eq!(result.matched_policies.len(), 1);
        let plan = &result.matched_policies[0];
        
        assert_eq!(plan.policy_name, "thermal-safety");
        assert_eq!(plan.status, PlanStatus::Violated);
        assert!(plan.computed_action.is_some());
    }

    #[test]
    fn test_plan_utilization_violation() {
        let yaml = r#"
        apiVersion: v1
        kind: EfficiencyProfile
        metadata:
          name: "test-profile-util"
          version: "1.0.0"
        selectors: {}
        policies:
          - name: "idle-check"
            target: gpu_utilization
            condition: "< 5%"
            severity: info
            action:
              type: lock_clock
        "#;
        
        let profile: EfficiencyProfile = serde_yaml::from_str(yaml).unwrap();
        let status = mock_snapshot();
        let result = profile.plan(&status);
        
        // Mock GPU util is 2.0, condition is < 5. This should be a violation (it IS idle).
        assert_eq!(result.matched_policies[0].status, PlanStatus::Violated);
    }
}
