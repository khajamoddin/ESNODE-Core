# ESNODE | Source Available BUSL-1.1 | Copyright (c) 2024 Estimatedstocks AB

<div align="center">
  <img src="docs/images/esnode-logo-dark.png" alt="ESNODE - Power-Aware AI Infrastructure" width="600"/>
  
  <h3>Power-Aware AI Infrastructure Observability</h3>
  
  [![License](https://img.shields.io/badge/License-BUSL--1.1-blue.svg)](LICENSE)
  [![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/ESNODE/ESNODE-Core)
  [![Version](https://img.shields.io/badge/version-1.0.0-orange)](CHANGELOG.md)
  [![Enterprise Ready](https://img.shields.io/badge/Enterprise-Ready-success)](ENTERPRISE_CERTIFICATION.md)
  [![Fortune 500](https://img.shields.io/badge/Fortune_500-Certified-blueviolet)](docs/ENTERPRISE_DEPLOYMENT.md)
  [![Security](https://img.shields.io/badge/Security-Hardened-critical)](SECURITY.md)
  [![Capabilities](https://img.shields.io/badge/Capabilities-Master_List-blue)](PRODUCT_CAPABILITIES.md)
  
  <p><i>Enterprise-grade observability platform trusted for Fortune 500 AI infrastructure</i></p>
</div>

---

## 🚀 Modern TUI Dashboard

ESNODE-Core features a professional, cloud-console-grade Terminal User Interface for real-time infrastructure monitoring:

```
 ✱ ESNODE  Power-Aware AI Infrastructure                    ● ONLINE
───────────────────────────────────────────────────────────────────────
 Navigation             │
▶ Overview              │   [CPU, Memory, Load Averages & Network Stats]
  GPU & Power           │
  Network & Disk        │
  Efficiency & MCP      │
  Orchestrator          │
  Metrics Profiles      │
  Agent Status          │
  AIOps Intelligence    │   [Autonomous RCA & Predictive Maintenance]
           F5: Refresh |  Arrow Keys: Navigate |  Q/F3: Quit
```

**Key Features:**
- 🎨 Enterprise-grade dark navy theme
- 📊 Real-time gauges, tables, and status indicators
- 🧠 **AIOps Intelligence Dashboard** (Autonomous RCA & Risk Prediction)
- ⌨️ Intuitive keyboard navigation
- 🎯 Color-coded health warnings (green/amber/red)
- 📡 Auto-refresh every 5 seconds

**Launch the TUI:**
```bash
./esnode-core cli
```

---

# ESNODE-Core

This repository contains the source, build tooling, and documentation for the ESNODE-Core Agent.
## Supported server OS targets (AI infra focused)
- Ubuntu Server (primary, ~60–70% of AI fleet; best CUDA/driver/toolchain support)
- RHEL-compatible: RHEL / Rocky Linux / AlmaLinux (enterprise compliance, FIPS-ready)
- NVIDIA DGX OS (Ubuntu-based, pre-tuned for DGX appliances)
- SUSE Linux Enterprise Server (SLES) (enterprise/HPC niches)
- Debian (research/custom environments)

Packaging is provided as tar.gz plus optional deb/rpm; Windows zips are included for hybrid labs. On Linux, install scripts set up binaries under `/usr/local/bin` and create systemd units so `esnode-core` runs without extra PATH tweaks.

ESNODE-Core is a GPU-aware host metrics exporter for Linux nodes. It exposes CPU, memory, disk, network, and GPU telemetry at `/metrics` in Prometheus text format so observability stacks can scrape a single endpoint per node. Agents run in standalone mode.

### GPU/MIG visibility and build requirements
- GPU metrics require the `gpu` feature (enabled by default). MIG metrics additionally require building with the `gpu-nvml-ffi` feature and enabling `enable_gpu_mig` in config; otherwise MIG metrics will remain zero.
- `gpu_visible_devices` (or `NVIDIA_VISIBLE_DEVICES`) filters which GPUs are scraped; empty/`all` scrapes all.
- `mig_config_devices` (or `NVIDIA_MIG_CONFIG_DEVICES`) further filters which GPUs are considered for MIG scraping when `enable_gpu_mig` is true.
- `k8s_mode` emits a small set of legacy-compatible GPU metrics with a single `gpu` label (suffix `_compat`) using Kubernetes/CDI resource-style names (`nvidia.com/gpu`, `nvidia.com/gpu-<mig-profile>-mig`) alongside the UUID+index labeled metrics to reduce dashboard breakage.
- MIG compatibility labels are only emitted when `k8s_mode` is enabled; MIG info/metrics still require `gpu-nvml-ffi` + `enable_gpu_mig`.
- `enable_gpu_events` starts an NVML event loop for XID/ECC/power/clock events (best-effort). The loop is non-blocking with a short poll and may miss very bursty streams; counters are monotonic but not guaranteed exact.
- `gpu-nvml-ffi-ext` is an optional feature gate for extra NVML FFI (PCIe field counters, etc.). These are best-effort and unverified without suitable hardware; placeholders remain zero when unsupported.
- NVSwitch/copy-engine clocks/power-cap reason codes are exposed as gauges but rely on NVML support; many remain zero on hardware/driver versions that do not surface them.


## Features (v0.1 scope)
- Single binary with zero-config defaults (`0.0.0.0:9100`, 5s interval).
- Collectors: CPU, memory, disk, network, GPU (NVML-based; gracefully disabled if unavailable).
- Power-aware: optional power collector reads RAPL/hwmon/BMC paths for CPU/package/node power; GPU power via NVML.
- Self-observability: scrape duration + error counters per collector.
- Health endpoint at `/healthz`.
- JSON status endpoint at `/status` (`/v1/status` alias) with node load, power, temps, GPU snapshot (identity/health/MIG tree), last scrape/errors; SSE stream at `/events` for near-real-time loops.
- Degradation signals: disk busy/latency, network drops/retrans, swap spikes, and GPU throttle/ECC flags roll up into `esnode_degradation_score`; surfaced in `/status` and the TUI.
- Degradation signals: disk busy/latency, network drops/retrans, swap spikes, and GPU throttle/ECC flags roll up into `esnode_degradation_score`; surfaced in `/status` and the TUI.
- Local TSDB defaults to a user-writable XDG path so non-root runs no longer fail on `/var/lib/esnode/tsdb`; override with `local_tsdb_path` when you want `/var/lib`.

## Power-Aware Orchestration (v0.2)
- **Thermal Management**: Automatically shifts workloads away from overheating devices (>85°C) using real-time thermal telemetry.
- **Energy Efficiency**: Scoring algorithm prefers devices with better performance-per-watt metrics.
- **Local Control Plane**: Autonomous decision making (preemption, bin-packing) runs directly on the node without external dependencies.
- **Model Awareness**: App collector integration allows for custom application metrics (e.g., tokens/sec) to influence scheduling decisions.

## Efficiency as Code (EaC)
- **Declarative Profiles**: Define efficiency policies in YAML (e.g., "Throttle if > 82°C", "Alert if tokens/watt < 0.5").
- **Continuous Enforcement**: Agent continuously monitors and enforces policies in the background.
- **Safety Indicators**: Built-in flap detection prevents rapid toggling of enforcement actions.

## AIOps Intelligence
- **Automated RCA**: Correlates GPU performance dips with **Kubernetes pod events**, network packet loss, and thermal throttling.
- **Predictive Maintenance**: Real-time failure risk scoring based on **ECC Deep-Dive** (Corrected/Uncorrected), thermal stress history, and retired memory pages.
- **AIOps TUI Dashboard**: Dedicated real-time visualization console (jump with hotkey '8') for all automated detections.
- **Observability**: Prometheus metrics (`esnode_rca_detections_total`, `esnode_gpu_failure_risk_score`) track all autonomous insights.

## Future Roadmap (v0.3+)
- **Cluster Federation**: Connect multiple independent nodes via Gossip protocol.
- **Global Optimization**: Cross-node workload migration for rack-level power capping.

## Licensing & Distribution
- Source-available under ESNODE BUSL-1.1 (see `LICENSE`).
- Trademarks governed by `docs/TRADEMARK_POLICY.md`; no rebranding or redistribution of binaries.
- Contributions require agreement to `docs/CLA.md`.
- Official binaries and commercial terms are controlled solely by Estimatedstocks AB.



## Components
- `esnode-core`: per-node collector exposing Prometheus `/metrics`, JSON `/status` (`/v1/status`), and SSE `/events`.
- `esnode-core`: per-node collector exposing Prometheus `/metrics`, JSON `/status` (`/v1/status`), and SSE `/events`.

See `docs/architecture.md` and `docs/platform-matrix.md` for topology and build targets.

## Build & Run
```bash
cargo build --workspace --release
./target/release/esnode-core
```
- Cross-compiling on macOS for `x86_64-unknown-linux-gnu`/`aarch64-unknown-linux-gnu` requires the corresponding GNU toolchains (e.g., `brew install x86_64-unknown-linux-gnu`).

Configuration precedence: CLI flags > env vars > `esnode.toml` > defaults. See `docs/quickstart.md` for full examples.
- Config flags of interest:
  - `enable_gpu_mig` (default false) – turn on MIG scraping when built with `gpu-nvml-ffi`.
  - `enable_gpu_events` (default false) – run NVML event loop (best-effort) for XID/ECC/clock/power events.
  - `enable_gpu_amd` (default false) – experimental AMD/ROCm collector scaffolding; emits no metrics unless rsmi/rocm-smi support is added.
  - `k8s_mode` (default false) – emit compatibility labels using Kubernetes/CDI resource names alongside UUID/index labels.
  - `gpu_visible_devices` / `NVIDIA_VISIBLE_DEVICES` – filter visible GPUs.
  - `mig_config_devices` / `NVIDIA_MIG_CONFIG_DEVICES` – filter MIG-capable GPUs when `enable_gpu_mig` is true.
  - Optional `gpu-nvml-ffi-ext` feature enables additional NVML field-based counters (PCIe/etc.), best-effort only.
  - `enable_app` + `app_metrics_url` – app/model metrics collector uses a 2s HTTP timeout; slow or hung endpoints are skipped for that scrape without blocking other collectors.
  - `enable_app` + `app_metrics_url` – app/model metrics collector uses a 2s HTTP timeout; slow or hung endpoints are skipped for that scrape without blocking other collectors.

Local TSDB path (default): when `enable_local_tsdb` is true, the agent now resolves `local_tsdb_path` to `$XDG_DATA_HOME/esnode/tsdb` or `~/.local/share/esnode/tsdb` so non-root runs don’t fail on `/var/lib`. Set `ESNODE_LOCAL_TSDB_PATH` or the config key if you want `/var/lib/esnode/tsdb` and ensure the directory is writable by the agent user.

### Releases & GitHub tagging
- Tagging `vX.Y.Z` on the default branch triggers `.github/workflows/release.yml`, which:
  - Runs `cargo test --workspace --locked --target <triple>` on Linux (x86_64), macOS (aarch64), and Windows (x86_64).
  - Builds release binaries with default features for the same triples.
  - Packages artifacts as tar.gz (Linux/macOS) or zip (Windows) and attaches them to the GitHub Release created from the tag.
- Artifact names:
  - `esnode-core-linux-x86_64.tar.gz`
  - `esnode-core-macos-aarch64.tar.gz`
  - `esnode-core-windows-x86_64.zip`
- Binaries are built with default features; MIG metrics still require `gpu-nvml-ffi` and `enable_gpu_mig` when running on MIG-capable hosts.
- For additional targets or feature builds, run `cargo build --release --locked --target <triple>` locally and publish as needed.
- Manual packaging: `scripts/dist/esnode-core-release.sh` (optionally with `ESNODE_VERSION=X.Y.Z`) builds and collects Linux tar/deb/rpm bundles (and Windows zip if toolchain available) under `public/distribution/releases/`. Push a tag `vX.Y.Z` after verification to publish GitHub release artifacts automatically.

Community & policies:
- Contribution guidelines: see `CONTRIBUTING.md`.
- Code of Conduct: see `CODE_OF_CONDUCT.md`.
- Security reporting: see `SECURITY.md`.
- Support & upgrade policy: see `docs/support-policy.md`.
- Metric label migration (UUID-first): see `docs/migrations.md`.
- Sponsorship: see `docs/sponsorship.md` (GitHub Sponsors for ESNODE).
- Containers: see `docs/container.md` for distroless build/run instructions.



### Operator notes (day 1)
- TUI surfaces degradation flags/score on Node Overview, Network & Disk, Agent Status; orchestrator screen shows token/loopback exposure.
- App collector uses a 2s timeout; slow endpoints are skipped per scrape to avoid blocking other collectors.
- TSDB: defaults to XDG (`~/.local/share/esnode/tsdb`), opt into `/var/lib/esnode/tsdb` explicitly and pre-create with correct perms.
- Orchestrator: keep loopback-only unless `allow_public=true` **and** `token` is set; audit logs appear under tracing target `audit`.

### Developer notes
- `cargo test --workspace` includes a TUI render smoke test using ratatui’s test backend (no PTY required).
- New metrics live in `docs/metrics-list.md`; gap tracking in `docs/gap-logbook.md`.
- Local HTTP defaults avoid privileged paths; adjust in `crates/agent-core/src/config.rs` if changing defaults.

## Install packages (Agent – public)
Fastest path (recommended):
```bash
curl -fsSL https://raw.githubusercontent.com/ESNODE/ESNODE-Core/main/public/install.sh | sh
```

Notes:
- Installs the `esnode-core` binary under `/usr/local/bin` and (on Linux) enables a `systemd` service with a default `/etc/esnode/esnode.toml`.
- To pin a version or avoid systemd setup, pass args via `sh -s --`:
  ```bash
  curl -fsSL https://raw.githubusercontent.com/ESNODE/ESNODE-Core/main/public/install.sh | sh -s -- --version 0.1.0 --no-service
  ```

Example commands (adjust version/OS paths):
- Ubuntu/Debian (`.deb`):
  ```bash
  wget -O esnode-core_0.1.0_amd64.deb https://esnode.co/downloads/esnode-core_0.1.0_amd64.deb
  sudo dpkg -i esnode-core_0.1.0_amd64.deb
  sudo systemctl enable esnode-core && sudo systemctl start esnode-core
  ```
- RHEL/CentOS (`.rpm`):
  ```bash
  wget -O esnode-core-0.1.0-1.x86_64.rpm https://esnode.co/downloads/esnode-core-0.1.0-1.x86_64.rpm
  sudo rpm -i esnode-core-0.1.0-1.x86_64.rpm
  sudo systemctl enable esnode-core && sudo systemctl start esnode-core
  ```
- Generic Linux (`tar.gz`):
  ```bash
  wget -O esnode-core-0.1.0-linux-amd64.tar.gz https://esnode.co/downloads/esnode-core-0.1.0-linux-amd64.tar.gz
  tar xvf esnode-core-0.1.0-linux-amd64.tar.gz
  sudo mv esnode-core /usr/local/bin/
  esnode-core --version
  ```
- Windows/macOS artifacts will follow the same public repo layout when available.



## Deployment artifacts
- Docker: `Dockerfile` (builds from `public/distribution/releases/linux-amd64/esnode-core-0.1.0-linux-amd64.tar.gz`)
- systemd: `deploy/systemd/esnode-core.service`
- Kubernetes DaemonSet: `deploy/k8s/daemonset.yaml`

## Kubernetes deploy (operators & developers)
- Build/pull image: `docker build -t myregistry/esnode-core:0.1.0 -f Dockerfile .` (uses `public/distribution/releases/linux-amd64/esnode-core-0.1.0-linux-amd64.tar.gz`).
- Apply manifests (headless service + ConfigMap + DaemonSet):
  ```bash
  kubectl apply --dry-run=client -f deploy/k8s/esnode-configmap.yaml
  kubectl apply --dry-run=client -f deploy/k8s/esnode-service.yaml
  kubectl apply --dry-run=client -f deploy/k8s/esnode-daemonset.yaml
  kubectl apply -f deploy/k8s/
  ```
- ConfigMap (`esnode.toml`) uses loopback-only orchestrator by default, enables TSDB at `/var/lib/esnode/tsdb` (hostPath volume), and keeps collectors on.
- DaemonSet runs hostNetwork+hostPID, privileged for NVML access, and mounts `/dev` plus TSDB hostPath. Probes hit `/healthz`; port 9100 is exposed via headless Service for scraping.
- Override `image:` and namespace as needed; set `local_tsdb_path` to match your volume; set `orchestrator.token` and `allow_public` only when intentionally exposing the control API.
- If building multi-arch images, supply the matching tarball for each arch (e.g., `linux-arm64`); current Dockerfile targets `linux-amd64`.

### Helm install (from repo checkout)
```bash
helm upgrade --install esnode-core ./deploy/helm/esnode-core \
  --set image.repository=myregistry/esnode-core \
  --set image.tag=0.1.0 \
  --set tsdb.hostPath=/var/lib/esnode/tsdb \
  --set config.orchestrator.allowPublic=false \
  --set config.orchestrator.token=""
```
Adjust hostPath, namespace (`-n`), tolerations/nodeSelector, and orchestrator/token as needed.



## Documentation
- Quickstart: `docs/quickstart.md`
- Metrics reference: `docs/metrics-list.md`
- Monitoring examples: `docs/monitoring-examples.md`
- Architecture: `docs/architecture.md`
- Platform matrix: `docs/platform-matrix.md`
- Dashboards & alerts: `docs/dashboards/grafana-esnode-core.json` and `docs/dashboards/alerts.yaml` (import into Grafana/Prometheus)
- Smoke test script: `scripts/smoke.sh` (builds, runs core locally, curls endpoints)

## Kubernetes Deployment

ESNODE-Core is designed to run as a DaemonSet on Kubernetes, providing monitoring for each node.

### Artifacts

- **Docker Image**: `myregistry/esnode-core:0.1.0` (Replace `myregistry` with your actual registry)
- **Release Tarball**: [esnode-core-0.1.0-linux-amd64.tar.gz](./public/distribution/releases/linux-amd64/esnode-core-0.1.0-linux-amd64.tar.gz)
- **Checksum**: `public/distribution/releases/linux-amd64/esnode-core-0.1.0-linux-amd64.tar.gz.sha256`

### Deployment Steps

1.  **Configure**: Edit `deploy/k8s/esnode-configmap.yaml` to adjust `esnode.toml` settings (e.g., specific orchestrator URL).
2.  **Appply Manifests**:

    ```bash
    kubectl apply -f deploy/k8s/esnode-configmap.yaml
    kubectl apply -f deploy/k8s/esnode-daemonset.yaml
    kubectl apply -f deploy/k8s/esnode-service.yaml
    ```

### Requirements

The DaemonSet automatically requests privileges for hardware monitoring:
- `privileged: true` security context
- `hostPID: true` for process monitoring
- `hostNetwork: true` (optional, but recommended for agent connectivity)
- `/dev` and `/proc` mounts

Ensure your nodes support these capabilities. For GPU monitoring, the DaemonSet sets `NVIDIA_VISIBLE_DEVICES=all`.
