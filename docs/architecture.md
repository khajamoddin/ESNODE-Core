ESNODE | Source Available BUSL-1.1 | Copyright (c) 2025 Estimatedstocks AB

# ESNODE Architecture (ESNODE-Core)

ESNODE-Core lives in this repository.

## Components
- `esnode-core`: per-node collector exposing:
  - `/metrics` Prometheus text (host + GPU + power + self-metrics)
  - `/status` and `/v1/status` JSON snapshot (load, power, temps, GPUs, last scrape/errors)
  - `/events` SSE stream of status snapshots (5s default)
  - `/healthz`
- `esnode-orchestrator`: optional autonomous resource manager (embedded lib, CLI-configurable) exposing:
  - `/orchestrator/metrics` JSON status

## Data Flow
1) Agent collectors gather host/GPU/power metrics on interval; publish to Prometheus + JSON snapshot + SSE.




## Notes
- NVLink counters and ECC are best-effort; set only when supported by NVML (ECC deltas emitted per GPU/type).
- Power envelope breach gauge (`esnode_node_power_envelope_exceeded`) relies on optional `--node-power-envelope-watts`.
- CPU package power/energy comes from RAPL where available; node power from hwmon/BMC/IPMI; GPU energy from NVML.
- KV cache fragmentation and tokens-per-watt/joule require app-side `model_*` metrics.

## Recommended Plan to Add IBM Support

- AIX
  - Investigate available Rust target support and libc portability; swap Linux filesystems with AIX APIs for CPU/mem/disk/net.
- z/OS
  - Assess Rust support and viable async/network stacks; use z/OS system calls and SMF for metrics if feasible.
- AS/400 (IBM i)
  - Use IBM i APIs (MI/OS APIs) for metrics; build with appropriate cross-compiler or native toolchain.
- General
  - Abstract collectors behind per-OS adapters; provide feature flags to compile the appropriate backends.
  - Replace NVML for GPU metrics or disable GPU metrics on non-NVIDIA platforms.
  - Add CI build matrix (GitHub Actions/enterprise CI) targeting Linux/macOS/Windows first, then IBM platforms when toolchains are validated.
  - Define packaging outputs under `public/distribution/<platform>` with OS-native installers.
