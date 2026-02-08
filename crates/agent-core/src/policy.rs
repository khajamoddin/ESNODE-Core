// ESNODE | Source Available BUSL-1.1 | Copyright (c) 2024 Estimatedstocks AB

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The root manifest for an Efficiency Profile.
/// Corresponds to the `kind: EfficiencyProfile` YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EfficiencyProfile {
    pub api_version: String,
    pub kind: String,
    pub metadata: ProfileMetadata,
    pub selectors: ProfileSelectors,
    pub policies: Vec<PolicyRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileSelectors {
    #[serde(default)]
    pub match_tags: HashMap<String, String>,
    #[serde(default)]
    pub match_labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub name: String,
    pub description: Option<String>,
    pub target: PolicyTarget,
    pub condition: String, // e.g., "> 80", "< 5%"
    #[serde(default)]
    pub duration: Option<String>, // e.g., "5m"
    pub action: PolicyAction,
    pub severity: PolicySeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyTarget {
    GpuTempCelsius,
    GpuUtilization,
    GpuPowerWatts,
    MemoryAllocatedPercent,
    TokensPerWatt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAction {
    #[serde(rename = "type")]
    pub action_type: ActionType,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    ThrottlePower,
    LockClock,
    Alert,
    KillProcess,
    MigratePod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicySeverity {
    Info,
    Warning,
    Critical,
}

/// The result of a `plan` operation.
#[derive(Debug, Clone, Serialize)]
pub struct PlanResult {
    pub profile_name: String,
    pub matched_policies: Vec<PolicyPlan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyPlan {
    pub policy_name: String,
    pub target_resource: String, // e.g., "GPU-0"
    pub current_value: String,
    pub threshold: String,
    pub status: PlanStatus,
    pub computed_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlanStatus {
    Satisfied,
    Violated,
    Skipped,
}

impl EfficiencyProfile {
    /// Simulates the profile against the current status snapshot (The "Plan" phase).
    pub fn plan(&self, status: &crate::state::StatusSnapshot) -> PlanResult {
        let mut plans = Vec::new();

        for policy in &self.policies {
            match policy.target {
                PolicyTarget::GpuTempCelsius => {
                    for gpu in &status.gpus {
                        let current = gpu.temperature_celsius.unwrap_or(0.0);
                        let (violated, limit) = check_condition(current, &policy.condition);
                        
                        let status_enum = if violated {
                            PlanStatus::Violated
                        } else {
                            PlanStatus::Satisfied
                        };

                        let action_desc = if violated {
                            Some(format!("Execute {:?} with params {:?}", policy.action.action_type, policy.action.parameters))
                        } else {
                            None
                        };

                        plans.push(PolicyPlan {
                            policy_name: policy.name.clone(),
                            target_resource: format!("GPU-{}", gpu.uuid.clone().unwrap_or(gpu.gpu.clone())),
                            current_value: format!("{:.1}C", current),
                            threshold: format!("{:.1}C", limit),
                            status: status_enum,
                            computed_action: action_desc,
                        });
                    }
                },
                PolicyTarget::GpuUtilization => {
                     for gpu in &status.gpus {
                        let current = gpu.util_percent.unwrap_or(0.0);
                         let (violated, limit) = check_condition(current, &policy.condition);
                        
                        let status_enum = if violated {
                            PlanStatus::Violated
                        } else {
                            PlanStatus::Satisfied
                        };
                         
                        plans.push(PolicyPlan {
                            policy_name: policy.name.clone(),
                            target_resource: format!("GPU-{}", gpu.uuid.clone().unwrap_or(gpu.gpu.clone())),
                            current_value: format!("{:.1}%", current),
                            threshold: format!("{:.1}%", limit),
                            status: status_enum,
                            computed_action: if violated { Some(format!("Action: {:?}", policy.action.action_type)) } else { None },
                        });
                     }
                }
                _ => {
                    // Placeholder for other metrics
                     plans.push(PolicyPlan {
                        policy_name: policy.name.clone(),
                        target_resource: "ALL".to_string(),
                        current_value: "N/A".to_string(),
                        threshold: policy.condition.clone(),
                        status: PlanStatus::Skipped,
                        computed_action: None,
                    });
                }
            }
        }

        PlanResult {
            profile_name: self.metadata.name.clone(),
            matched_policies: plans,
        }
    }
}

/// Rudimentary parser for conditions like "> 80" or "< 5".
/// Returns (is_violated, threshold_value).
fn check_condition(current: f64, condition: &str) -> (bool, f64) {
    let parts: Vec<&str> = condition.split_whitespace().collect();
    if parts.len() < 2 {
        return (false, 0.0);
    }
    
    let op = parts[0];
    let val_str = parts[1].replace(['%', 'C'], ""); // strip units
    let threshold = val_str.parse::<f64>().unwrap_or(0.0);

    let violated = match op {
        ">" => current > threshold,
        ">=" => current >= threshold,
        "<" => current < threshold,
        "<=" => current <= threshold,
        "=" | "==" => (current - threshold).abs() < f64::EPSILON,
        "!=" => (current - threshold).abs() > f64::EPSILON,
        _ => false,
    };

    (violated, threshold)
}
