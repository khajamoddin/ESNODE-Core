ESNODE | Source Available BUSL-1.1 | Copyright (c) 2025 Estimatedstocks AB

# Platform & Build Matrix (2025)

Primary targets (AI infra focus):
- Ubuntu Server (20.04/22.04/24) — primary CUDA/driver path.
- RHEL / Rocky / Alma (8/9) — enterprise compliance.
- NVIDIA DGX OS (Ubuntu-based) — DGX appliances.
- SLES 15 SPx — enterprise/HPC niches.
- Debian — research/custom.
- Windows Server (optional hybrid labs; GPU/power collectors degrade gracefully).

Hypervisors:
- VMware ESXi 9.x, KVM (kernel 6.x), Hyper-V (Server 2025), Proxmox VE 8/9.

Notes:
- GPU features (NVML/NVLink/ECC) require NVIDIA drivers + NVML present on host/container.
- Power collectors: RAPL/hwmon/BMC/IPMI availability varies by hardware/firmware; envelope flag requires `--node-power-envelope-watts`.
- Service managers: systemd unit provided for Linux; Windows installer scripts provided (NSSM). macOS/launchd not shipped.
- Binaries: `esnode-core` (in this repo) is built per target OS/arch; `esnode-pulse` ships separately as a licensed controller. Ensure OpenSSL/Rustls compatibility on chosen platform.
