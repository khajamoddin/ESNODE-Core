# Changelog

All notable changes to this project will be documented here.

## Release notes & policy
- We follow semver: MAJOR for breaking config/CLI/metrics/labels, MINOR for additive, PATCH for fixes.
- Deprecations are announced here and in release notes; removals occur only in the next MAJOR when possible.
- Metric/label changes must include migration notes and, where feasible, compatibility labels during a deprecation window.
- Platform validation updates (GPU/driver/CUDA/K8s) should be recorded here and in `docs/platform-matrix.md`.
- UUID-first metrics: see `docs/migrations.md` for guidance. `_compat` metrics (when `k8s_mode` is enabled) are provided for a transition period; plan to move dashboards to UUID-based labels.

## [v0.2.0] - 2026-02-07
### Added
- **Brand Identity Update**: New ESNODE logo with asterisk symbol (✱) and tagline "Power-Aware AI Infrastructure"
- **TUI Modernization**: Complete dashboard redesign with cloud-provider-grade interface
  - Dark navy theme matching brand guidelines (RGB 26,35,50)
  - Professional color palette using Material Design colors
  - Enhanced sidebar navigation with bright blue active selection
  - Improved status indicators with colored bullets (●)
  - All 7 screens redesigned with modern widgets (gauges, tables, lists)
- **Documentation Suite**:
  - Brand guidelines (`docs/BRAND_GUIDELINES.md`) with complete color palette and design system
  - TUI user guide (`docs/TUI_USER_GUIDE.md`) with comprehensive screen documentation
  - Branding update summary (`BRANDING_UPDATE_SUMMARY.md`)
  - TUI testing report (`TUI_TESTING_REPORT.md`)
- **Logo Assets**: Added high-quality logo files to `docs/images/`
  - `esnode-logo-dark.png` - Horizontal logo with tagline
  - `esnode-icon.png` - Square icon for avatars
- **README Enhancement**: Updated with logo, TUI preview, and professional badges

### Changed
- **TUI Color Scheme**: Migrated from basic terminal colors to professional Material Design palette
  - Header: Blue → Dark Navy (26, 35, 50)
  - Sidebar: Black → Darker Navy (20, 27, 40)
  - Active Selection: Cyan → Bright Blue (37, 99, 235)
  - Labels: Cyan → Light Blue (100, 181, 246)
  - Success: Green → Material Green (76, 175, 80)
  - Warning: Yellow → Amber (255, 193, 7)
  - Error: Red → Material Red (244, 67, 54)
- **TUI Header**: Updated from "Estimatedstocks AB | Managed AI Infrastructure" to "✱ ESNODE  Power-Aware AI Infrastructure"
- **Status Indicators**: Enhanced with bullet symbols (● ONLINE, ● CONNECTING...)

### Improved
- **TUI Layout**: Better visual hierarchy with consistent spacing and borders
- **Accessibility**: Color-blind friendly palette with high contrast
- **Documentation**: Comprehensive guides for all screens and features
- **Professional Presentation**: Enterprise-grade quality matching AWS/Azure/GCP consoles

## Unreleased

### Added - AIOps & Predictive Maintenance (2026-02-09)
- **Automated Root Cause Analysis (RCA)**: Real-time correlation of GPU performance dips with network packet loss and thermal throttling events
  - New module: `crates/agent-core/src/rca.rs`
  - Metric: `esnode_rca_detections_total` with labels `cause` and `confidence`
  - Sliding window analysis (5-minute default) with autonomous detection every scrape cycle
- **Predictive Maintenance Engine**: ML-based failure risk scoring for GPUs
  - New module: `crates/agent-core/src/predictive.rs`
  - Metric: `esnode_gpu_failure_risk_score` (0-100 scale) per GPU UUID
  - Risk factors: Uncorrected ECC errors, corrected ECC rate trends, thermal throttling frequency, memory page retirement
  - Automatic alerting when risk score >= 50.0
- **Enhanced GPU Telemetry**: Extended `GpuHealth` with aggregate ECC error counters
- **Policy Metrics**: Added `esnode_policy_violations_total` and `esnode_policy_enforced_total` for Efficiency as Code observability

### Fixed
- Fixed local TSDB default path for non-root runs by resolving to `$XDG_DATA_HOME/esnode/tsdb` or `~/.local/share/esnode/tsdb`, and now disable with a clear warning if initialization fails (resolves GitHub issue #2). Documented upgrade guidance in README and quickstart.
- App collector now uses async HTTP with a 2s timeout to avoid blocking the scrape loop; CLI client uses a lightweight HTTP helper with timeouts to keep status/metrics fetches non-blocking even when endpoints hang.
- TSDB export now snapshots the current block without closing it, preventing index resets and missing samples when exporting mid-window.
- Orchestrator control API is loopback-only by default; set `orchestrator.allow_public=true` to expose `/orchestrator/*` on non-loopback listeners and `orchestrator.token` to require bearer auth.
- Added swap/disk/network degradation flags, aggregate degradation score, and audit logging on orchestrator actions; created a living gap logbook (`docs/gap-logbook.md`).
- Add GitHub Actions release pipeline, packaging via `scripts/dist/esnode-core-release.sh`, and artifact checksums.
- Expanded GPU/MIG telemetry, K8s compatibility labels, and NVML FFI scaffolding.
- Added contributor documentation (CONTRIBUTING.md, CODE_OF_CONDUCT.md) and security policy.

## [v0.1.0] - 2024-xx-xx
- Initial public source-available release of ESNODE-Core (BUSL-1.1).

_Fill in dated sections when tagging releases._

