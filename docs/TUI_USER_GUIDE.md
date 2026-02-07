# ESNODE TUI User Guide

<div align="center">
  <img src="images/esnode-logo-dark.png" alt="ESNODE" width="500"/>
</div>

---

## Overview

The ESNODE Terminal User Interface (TUI) provides real-time infrastructure monitoring through an intuitive, keyboard-driven dashboard. Designed to match the quality of cloud provider consoles (AWS, Azure, GCP), it offers comprehensive visibility into GPU clusters, power consumption, and system health.

---

## Quick Start

### Launch the TUI

```bash
# From the ESNODE-Core directory
./target/release/esnode-core cli

# Or if installed system-wide
esnode-core cli
```

### Prerequisites
- **Agent Daemon:** Must be running (`esnode-core daemon`)
- **Terminal:** Modern terminal with Unicode and color support
- **Minimum Size:** 80x24 characters (recommended: 120x40)

---

## Interface Layout

```
┌──────────────────────────────────────────────────────────────────┐
│ ✱ ESNODE  Power-Aware AI Infrastructure         ● ONLINE        │ 1. Header
├────────────┬─────────────────────────────────────────────────────┤
│            │                                                     │
│ Navigation │  Content Area                                       │ 2. Body
│            │  [Gauges, Tables, Charts]                           │
│ ▶ Overview │                                                     │
│  GPU Power │                                                     │ 3. Sidebar
│  Network   │                                                     │
│  ...       │                                                     │
│            │                                                     │
├────────────┴─────────────────────────────────────────────────────┤
│ F5: Refresh | Arrow Keys: Navigate | Q/F3: Quit | Mode: Color  │ 4. Footer
└──────────────────────────────────────────────────────────────────┘
```

### Components

1. **Header Bar**
   - ESNODE logo with asterisk symbol (✱)
   - "Power-Aware AI Infrastructure" tagline
   - Connection status indicator (● ONLINE / ● CONNECTING...)

2. **Sidebar Navigation** (Left, 25% width)
   - List of available screens
   - Active screen highlighted in blue
   - Selection arrow (▶) indicator

3. **Content Area** (Right, 75% width)
   - Screen-specific widgets (gauges, tables, lists)
   - Real-time data updates every 5 seconds
   - Color-coded health indicators

4. **Footer Bar**
   - Keyboard shortcut reference
   - Current mode indicator (Color / B&W)

---

## Keyboard Controls

### Navigation
| Key | Action |
|-----|--------|
| `↑` / `↓` | Previous / Next screen |
| `1-7` | Jump to specific screen (legacy hotkeys) |
| `F5` | Manual refresh |
| `Q` / `ESC` / `F3` | Quit TUI |

### Screen Mapping
| Number | Screen |
|--------|--------|
| `1` | Overview |
| `2` | GPU & Power |
| `3` | Network & Disk |
| `4` | Efficiency & MCP |
| `5` | Metrics Profiles |
| `6` | Agent Status |
| `7` | Orchestrator |

---

## Screens Reference

### 1. Overview Screen

**Purpose:** High-level system health dashboard

**Widgets:**
- **CPU Usage Gauge:** Real-time CPU utilization (0-100%)
  - Green: Normal (<80%)
  - Red: High load (≥80%)
  
- **Memory Usage Gauge:** RAM consumption with GB display
  - Shows: Used / Total GB + Percentage
  
- **System Health Box:**
  - Disk health status (OK / DEGRADED)
  - Swap health status (OK / DEGRADED)
  - System uptime (days, hours, minutes)

- **System Details Table:**
  - Load averages (1m, 5m, 15m)
  - Network Rx/Tx rates
  - Network interface name

**Example:**
```
┌─ Overview ──────────────────────────────────────────┐
│                                                      │
│ ┌─ CPU Usage ──┐ ┌─ Memory Usage ─┐ ┌─ Health ────┐ │
│ │ ████████░░ 52%│ │ ██████████  73%│ │ Disk:  OK  │ │
│ │              │ │ 58/80 GB       │ │ Swap:  OK  │ │
│ └──────────────┘ └────────────────┘ │ Up: 3d 12h │ │
│                                     └────────────┘ │
│                                                      │
│ ┌─ System Details ──────────────────────────────────┐ │
│ │ Load Avg (1m)  │ 3.24                           │ │
│ │ Load Avg (5m)  │ 2.89                           │ │
│ │ Load Avg (15m) │ 2.45                           │ │
│ │ Network Rx     │ 2.4 GiB/s (eth0)               │ │
│ │ Network Tx     │ 1.8 GiB/s                      │ │
│ └────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

---

### 2. GPU & Power Screen

**Purpose:** Detailed GPU telemetry and power metrics

**Data Columns:**
- **ID:** GPU index (0-N)
- **Util:** Utilization percentage
- **Mem Used:** VRAM consumption (MB)
- **Power:** Current power draw (Watts)
- **Temp:** Temperature (°C)

**Color Coding:**
- **Normal:** White text (temp <80°C)
- **Warning:** Yellow text (temp 80-85°C)
- **Critical:** Red text (temp >85°C)

**Example:**
```
┌─ GPU Telemetry ─────────────────────────────────────┐
│ ID    │ Util   │ Mem Used  │ Power    │ Temp      │
│────────────────────────────────────────────────────│
│ GPU 0 │ 85.2%  │ 42156 MB  │ 285.3 W  │ 72.0°C    │
│ GPU 1 │ 92.1%  │ 45823 MB  │ 310.5 W  │ 78.0°C ⚠  │
│ GPU 2 │ 67.8%  │ 31245 MB  │ 245.8 W  │ 65.0°C    │
│ GPU 3 │ 88.4%  │ 38567 MB  │ 295.1 W  │ 70.0°C    │
│ GPU 4 │ 91.2%  │ 44012 MB  │ 305.7 W  │ 76.0°C    │
│ GPU 5 │ 73.5%  │ 35678 MB  │ 265.3 W  │ 68.0°C    │
│ GPU 6 │ 85.9%  │ 41234 MB  │ 288.4 W  │ 71.0°C    │
│ GPU 7 │ 79.3%  │ 37890 MB  │ 275.2 W  │ 69.0°C    │
└─────────────────────────────────────────────────────┘
```

---

### 3. Network & Disk Screen

**Purpose:** Network and storage health monitoring

**Sections:**

1. **Health Status**
   - Disk health (OK / DEGRADED)
   - Network health (OK / DEGRADED)
   - Swap health (OK / DEGRADED)

2. **Network Interface Table**
   - Rx Rate (bytes/sec)
   - Tx Rate (bytes/sec)  
   - Dropped packets count

3. **Storage / Disk Table**
   - Disk usage (used / total)
   - I/O latency (ms)
   - Swap usage

---

### 4. Efficiency & MCP Screen

**Purpose:** Energy efficiency metrics and AI workload performance

**Metrics:**
- **Tokens per Joule:** Energy efficiency for inference
- **Tokens per Watt:** Power efficiency for inference
- **Node Power Draw:** Total node power consumption (W)
- **Avg GPU Util:** Average GPU utilization across devices
- **Avg GPU Power:** Average GPU power draw (W/GPU)
- **CPU Util:** CPU utilization percentage

**Note:** Requires application metrics integration for tokens/sec data.

---

### 5. Metrics Profiles Screen

**Purpose:** Configuration status for metric collectors

**Toggles:**
- **[Y/N]** Host / Node Metrics (CPU, memory, disk, network)
- **[Y/N]** GPU Core Metrics (utilization, memory)
- **[Y/N]** GPU Power & Energy (power, temperature)
- **[Y/N]** MCP Efficiency (tokens/watt metrics)
- **[Y/N]** App / HTTP Metrics (application-level metrics)
- **[Y/N]** Rack Thermals (thermal sensors)

**Status:**
- `[Y]` = Enabled and collecting
- `[N]` = Disabled

---

### 6. Agent Status Screen

**Purpose:** Agent health and error monitoring

**Sections:**

1. **Agent Health Table**
   - Running status (YES / WARN)
   - Last scrape timestamp (Unix ms)
   - Degradation score (0-100)

2. **Recent Errors List**
   - Collector name
   - Error message
   - Timestamp

**Example:**
```
┌─ Agent Health ──────────────────────────────────────┐
│ Agent Running         │ YES                         │
│ Last Scrape Time      │ 1707294000000               │
│ Degradation Score     │ 12                          │
└─────────────────────────────────────────────────────┘

┌─ Recent Errors (3) ─────────────────────────────────┐
│ [gpu] Failed to read temp sensor (ts=1707293950)    │
│ [disk] I/O timeout on /dev/nvme0 (ts=1707293912)    │
│ [network] High packet loss detected (ts=1707293845) │
└─────────────────────────────────────────────────────┘
```

---

### 7. Orchestrator Screen

**Purpose:** Power-aware orchestration status

**Information:**
- **Autonomy Mode:** Status (ACTIVE / INACTIVE)
- **Scheduling:** Power-aware scheduling enabled/disabled
- **Description:** Overview of orchestrator capabilities

**Example:**
```
┌─ Orchestrator Status ───────────────────────────────┐
│ Autonomy Mode: ● ACTIVE                             │
│                                                      │
│ Orchestrator is running autonomously on this node.  │
│ Power-aware scheduling is enabled.                  │
│                                                      │
│ Features:                                            │
│ • Thermal management (device temp >85°C avoidance)  │
│ • Energy efficiency scoring (tokens/watt)           │
│ • Local control plane (autonomous decisions)        │
└──────────────────────────────────────────────────────┘
```

---

## Color Scheme

### Status Indicators
- **Green (● / ✓):** Healthy, OK, Success
- **Amber/Orange (● / ⚠):** Warning, Connecting, Caution
- **Red (● / ✗):** Critical, Error, Failure

### UI Elements
- **Light Blue:** Labels, headers, secondary text
- **White:** Primary text, data values
- **Dark Navy:** Header background (#1a2332)
- **Darker Navy:** Sidebar background (#141b28)
- **Bright Blue:** Active selection (#2563eb)

### B&W Mode
To enable black-and-white mode (for terminals without color support):
```bash
esnode-core cli --no-color
```

---

## Troubleshooting

### Issue: "Waiting for data from agent daemon..."

**Cause:** Agent daemon is not running or not reachable

**Solutions:**
1. Start the daemon: `esnode-core daemon`
2. Check daemon status: `systemctl status esnode-core`
3. Verify port 9100 is listening: `netstat -tuln | grep 9100`

### Issue: Garbled or missing characters

**Cause:** Terminal doesn't support Unicode

**Solutions:**
1. Use a modern terminal emulator (e.g., iTerm2, Windows Terminal, GNOME Terminal)
2. Enable UTF-8 encoding in terminal settings
3. Use `--no-color` flag for simplified ASCII output

### Issue: Screen too small

**Cause:** Terminal window is too narrow

**Solution:** Resize terminal to at least 80x24 (recommended 120x40)

---

## Advanced Usage

### Keyboard Shortcuts Reference

**Full Shortcut List:**
```
Global:
  F3, ESC, q   : Quit
  F5           : Manual refresh
  ↑            : Previous screen
  ↓            : Next screen

Quick Jump (Legacy):
  1            : Overview
  2            : GPU & Power
  3            : Network & Disk
  4            : Efficiency
  5            : Metrics Profiles
  6            : Agent Status
  7            : Orchestrator
```

### Auto-Refresh Behavior
- **Interval:** 5 seconds
- **On Connection Loss:** Status changes to "● CONNECTING..."
- **On Reconnect:** Data resumes automatically

### Screen Cycling
- Use `↑` and `↓` to cycle through screens in order
- Navigation wraps around (from last to first screen)

---

## Tips & Best Practices

1. **Monitor in Background:** Use `screen` or `tmux` to keep TUI running
2. **Color Accessibility:** Use `--no-color` for better visibility in some terminals
3. **Regular Checks:** Review "Agent Status" screen for errors
4. **GPU Thermal:** Watch GPU temperatures in GPU screen (⚠ appears at 80°C)
5. **Power Efficiency:** Check Efficiency screen for workload optimization

---

## Integration with Agent

The TUI connects to the local agent daemon via HTTP:
- **Endpoint:** `http://localhost:9100/status`
- **Authentication:** None (localhost only)
- **Protocol:** REST API with JSON responses

---

## Screenshots

### Overview Screen
*Real-time dashboard showing CPU, memory, and system health*

### GPU & Power Screen
*Detailed GPU telemetry with thermal warnings*

### Orchestrator Screen
*Power-aware orchestration status and capabilities*

---

## Version Information

**Current Version:** v0.2.0  
**Release Date:** 2026-02-07  
**Compatible Agent:** ESNODE-Core v0.2+  

---

## Support

**Documentation:** [GitHub Repository](https://github.com/ESNODE/ESNODE-Core)  
**Issues:** [Report Bugs](https://github.com/ESNODE/ESNODE-Core/issues)  
**License:** BUSL-1.1 (Source Available)  

---

**Maintained by the ESNODE Team**  
*Last Updated: 2026-02-07*
