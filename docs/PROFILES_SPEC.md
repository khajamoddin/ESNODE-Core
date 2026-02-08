# ESNODE Efficiency Profile Specification (v0.1)

This document defines the schema for **ESNODE Efficiency Profiles** (`.es` or `.esnode.yaml`). These profiles act as the "source of truth" for how AI infrastructure should behave, transforming manual tuning into **Efficiency as Code (EaC)**.

## 1. Overview

An Efficiency Profile is a declarative manifest that defines:
1.  **Targets:** Which hardware resources (GPUs, Nodes) are being managed.
2.  **Constraints:** The safety and performance boundaries (e.g., Max Temp, Min Utilization).
3.  **Actions:** What ESNODE should do when constraints are violated or optimized (e.g., Throttle Power, Alert).

**File Extension:** `.es` or `.yaml`

---

## 2. Schema Structure

A valid ESNODE profile consists of three main blocks: `metadata`, `selectors`, and `policies`.

```yaml
apiVersion: v1
kind: EfficiencyProfile
metadata:
  name: "llama3-training-h100"
  description: "High-performance profile for Llama 3 training on H100s. Prioritizes throughput over power saving."
  version: "1.0.0"

# SELECTORS: Who does this apply to?
selectors:
  matchTags:
    gpu_model: "H100"
    cluster_zone: "us-east-1"
  matchLabels:
    workload_type: "training"

# POLICIES: The rules of the road
policies:
  - name: "thermal-safety"
    description: "Prevent GPU overheating to avoid hardware damage."
    target: "gpu_temp_celsius"
    condition: "> 82"
    action:
      type: "throttle_power"
      parameters:
        step_watts: 50
        min_watts: 300
    severity: "critical"

  - name: "idle-power-save"
    description: "Aggressively downclock if GPU is idle for >5 mins."
    target: "gpu_utilization"
    condition: "< 5%"
    duration: "5m"
    action:
      type: "lock_clock"
      parameters:
        frequency_mhz: 210  # Idle clock
    severity: "info"

  - name: "efficiency-enforcement"
    description: "Ensure we are getting good tokens/watt."
    target: "tokens_per_watt"
    condition: "< 0.5"
    action:
      type: "alert"
      channel: "slack-devops"
```

---

## 3. Core Concepts

### 3.1 Selectors
Selectors allow a single profile to be applied to a heterogeneous fleet.
*   `matchTags`: Matches hardware properties discovered by ESNODE (e.g., `gpu_model`, `vram_size`, `driver_version`).
*   `matchLabels`: Matches logical labels applied by the orchestrator or user (e.g., `env=prod`, `team=research`).

### 3.2 Policy Definition
Each policy rule has:
*   **Target:** The metric to observe (e.g., `gpu_power_watts`, `memory_allocated_percent`).
*   **Condition:** The usage threshold (Standard boolean operators: `>`, `<`, `=`, `!=`).
*   **Duration:** (Optional) How long the condition must persist before triggering (debouncing).
*   **Action:** The remediation step.

### 3.3 Action Types
The `action` block defines the "Actuator" logic.

| Action Type | Description | Parameters |
| :--- | :--- | :--- |
| `throttle_power` | Reduces the GPU power limit (PL). | `step_watts` (decrease amount), `min_watts` (floor). |
| `lock_clock` | Locks the GPU graphics clock to a specific frequency. | `frequency_mhz`. |
| `alert` | Sends a notification without taking action. | `channel` (webhook/integration name). |
| `kill_process` | Terminates the process consuming the resource (Safety constraint). | `grace_period_seconds`. |
| `migrate_pod` | (K8s only) Signals the scheduler to drain the node. | `node_condition`. |

---

## 4. The Workflow: generic-iac-workflow

Just like standard IaC tools, ESNODE uses a state-based workflow.

### 4.1 Plan (`esnode plan`)
Simulates the profile against the *current* real-time metrics of the node.

*   *User Input:* `esnode plan -f training-profile.yaml`
*   *Output:*
    ```text
    Refreshing state...
    GPU-0 (H100) State: Temp=78C, Power=650W, Util=99%

    Plan: 1 policy to enforce.
    
    [+] Policy "thermal-safety":
        Current Temp (78C) is approaching Limit (82C).
        Status: MONITORING (No action needed yet).
    
    [+] Policy "idle-power-save":
        Current Util (99%) > Threshold (5%).
        Status: SKIPPED.
    ```

### 4.2 Enforce (`esnode apply` / `esnode enforce`)
Applies the profile to the active Agent/Orchestrator. The agent then enters a "Control Loop" where it continuously checks these policies.

*   *User Input:* `esnode enforce -f training-profile.yaml`
*   *Output:*
    ```text
    Profile 'train-h100' applied to Node-01.
    Active Control Loop started.
    ```

---

## 5. Future Extensions

*   **Drift Detection:** `esnode scan` will check if the hardware state has "drifted" from the profile (e.g., if a user manually reset power limits via `nvidia-smi`, ESNODE detects the diff and corrects it).
*   **Profile Registry:** A public hub of verified profiles for common models (Llama-3, SDXL, Bert) and hardware (H100, A100, T4).

