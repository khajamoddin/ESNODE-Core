# ESNODE-Core: Product Capabilities v1.0

ESNODE-Core is the industry's first **Power-Aware AIOps Infrastructure Observability** framework designed exclusively for high-performance AI/ML clusters. It combines deep hardware telemetry with autonomous intelligent operations to ensure maximum GPU availability and energy efficiency.

---

## 🚀 1. Autonomous AIOps Intelligence
ESNODE-Core doesn't just monitor—it understands. Our built-in RCA engine and predictive models move your infrastructure from reactive to proactive.

### **Autonomous Root Cause Analysis (RCA)**
- **Kubernetes Event Correlation**: Automatically maps GPU performance degradation to Kubernetes pod events (evictions, rescheduling, startup spikes).
- **Multi-Factor Correlation**: Simultaneously analyzes thermal throttling, power capping, and network congestion to identify the true bottleneck.
- **Confidence Scoring**: Every RCA event includes a confidence metric (0-100%) to assist SRE teams in prioritizing manual intervention.

### **Predictive Maintenance**
- **ECC/Thermal Deep-Dive**: Monitors both Corrected and Uncorrected ECC errors alongside historical thermal stress to predict imminent GPU failure.
- **Risk Assessment Dashboard**: Real-time risk scoring for every GPU in the cluster.
- **Proactive Alerts**: Notifies management systems before a "Self-Healing" event is triggered, allowing for graceful workload migration.

---

## ⚖️ 2. Power-Aware Orchestration (Efficiency as Code)
Optimize your AI infrastructure for sustainability and cost without compromising on performance.

### **Thermal-Driven Scheduling Visibility**
- **Dynamic Load Balancing**: Real-time visibility into how workloads should be distributed based on device temperatures (>85°C avoidance).
- **Performance-per-Watt Profiling**: Identify which GPUs are delivering the highest throughput for the least power at any given moment.

### **EaC (Efficiency as Code) Policy Engine**
- **Declarative YAML Profiles**: Define high-level efficiency goals (e.g., "Max Power 250W if Load < 50%").
- **Continuous Enforcement**: The agent autonomously applies clock-limiting or power-capping actions to keep the node within its efficiency envelope.
- **Audit Logging**: Complete traceability of all autonomous actions taken by the local control plane.

---

## 📊 3. Full-Stack AI Visibility
Comprehensive observability across the entire AI compute stack.

### **GPU & MIG Specialization**
- **Multi-Instance GPU (MIG) Tree**: Full visualization of MIG slices, profiles, and placement.
- **Fabric & Link Telemetry**: Real-time monitoring of NVLink and PCIe bandwidth, errors, and link generations.
- **Accelerator Health**: Deep introspection into VRAM, SM utilization, Decoders, Encoders, and HW/SW throttle reasons.

### **Host-Level Context**
- **NUMA-Aware Monitoring**: CPU, memory, and interrupt mapping across NUMA domains (essential for low-latency AI training).
- **Storage & Network Degradation**: Intelligent detection of I/O wait spikes or TCP retransmissions that impact training speed.

---

## 🖥️ 4. Enterprise-Grade Operations
Designed for security, scale, and ease of use in production environments.

### **Modern TUI Dashboard**
- **Cloud-Console Grade UI**: A professional terminal interface for high-density monitoring without needing a web browser.
- **Dedicated AIOps View (Hotkey '8')**: Instant access to real-time RCA detections and predictive risk scores.
- **One-Shot Diagnostics (Hotkey '7')**: Rapid node health check for quick troubleshooting.

### **Production-Ready Extras**
- **Zero-Dependency Core**: Compiled in Rust for maximum memory safety and performance.
- **Local TSDB Buffer**: Resilient JSONL-backed telemetry buffer ensures no data is lost during network partitions.
- **Prometheus-Native**: First-class support for Prometheus scraping with curated Grafana dashboards.
- **Secure by Default**: mTLS support, signed binaries, and non-root execution capabilities.

---

## 📦 Distribution Formats
- **Enterprise Binaries**: Signed, optimized binaries for Linux (x86_64, aarch64), macOS, and Windows.
- **Standard Linux Packages**: `.deb` and `.rpm` for major distributions (Ubuntu, RHEL, Rocky, Alma).
- **Containerized**: Hardened Alpine-based container images for Kubernetes/Docker deployments.
- **Helm Charts**: One-click deployment for GPU-accelerated Kubernetes clusters.

---
© 2024 Estimatedstocks AB | ESNODE-Core is distributed under the BUSL-1.1 license.
