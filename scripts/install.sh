#!/usr/bin/env bash
# ESNODE-Core Installation Script
# Version: 0.2.0
# Copyright (c) 2024 Estimatedstocks AB

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Banner
echo -e "${CYAN}"
cat << "EOF"
  ✱ ESNODE
  Power-Aware AI Infrastructure
  Installation Script v0.2.0
EOF
echo -e "${NC}"

# Configuration
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
SERVICE_USER="${SERVICE_USER:-esnode}"
SERVICE_DIR="/etc/systemd/system"
CONFIG_DIR="/etc/esnode"
DATA_DIR="/var/lib/esnode"
LOG_DIR="/var/log/esnode"

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
  echo -e "${YELLOW}⚠ This script should be run as root for system-wide installation${NC}"
  echo -e "${YELLOW}  For user installation, set: INSTALL_DIR=\$HOME/.local/bin${NC}"
  read -p "Continue anyway? (y/N): " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
  fi
  USER_INSTALL=true
else
  USER_INSTALL=false
fi

# Detect OS and architecture
detect_platform() {
  OS=$(uname -s | tr '[:upper:]' '[:lower:]')
  ARCH=$(uname -m)
  
  case "$ARCH" in
    x86_64)
      ARCH="amd64"
      ;;
    aarch64|arm64)
      ARCH="arm64"
      ;;
    *)
      echo -e "${RED}✗ Unsupported architecture: $ARCH${NC}"
      exit 1
      ;;
  esac
  
  case "$OS" in
    linux|darwin)
      echo -e "${GREEN}✓ Detected platform: $OS-$ARCH${NC}"
      ;;
    *)
      echo -e "${RED}✗ Unsupported OS: $OS${NC}"
      exit 1
      ;;
  esac
}

# Check for required binary
check_binary() {
  if [ -f "esnode-core" ]; then
    BINARY_PATH="esnode-core"
  elif [ -f "./target/release/esnode-core" ]; then
    BINARY_PATH="./target/release/esnode-core"
  else
    echo -e "${RED}✗ esnode-core binary not found${NC}"
    echo "  Please ensure you're in the correct directory or build the project first"
    exit 1
  fi
  
  # Verify binary works
  if ! "$BINARY_PATH" --version &>/dev/null; then
    echo -e "${RED}✗ Binary verification failed${NC}"
    exit 1
  fi
  
  VERSION=$("$BINARY_PATH" --version | grep -oP 'v\d+\.\d+\.\d+' || echo "unknown")
  echo -e "${GREEN}✓ Found esnode-core $VERSION${NC}"
}

# Install binary
install_binary() {
  echo -e "${BLUE}==>${NC} Installing binary to $INSTALL_DIR"
  
  mkdir -p "$INSTALL_DIR"
  cp "$BINARY_PATH" "$INSTALL_DIR/esnode-core"
  chmod +x "$INSTALL_DIR/esnode-core"
  
  echo -e "${GREEN}✓ Binary installed${NC}"
}

# Create service user (Linux only, system install)
create_service_user() {
  if [ "$USER_INSTALL" = true ] || [ "$OS" != "linux" ]; then
    return 0
  fi
  
  if id "$SERVICE_USER" &>/dev/null; then
    echo -e "${YELLOW}⚠ User $SERVICE_USER already exists${NC}"
  else
    echo -e "${BLUE}==>${NC} Creating service user: $SERVICE_USER"
    useradd --system --no-create-home --shell /bin/false "$SERVICE_USER"
    echo -e "${GREEN}✓ Service user created${NC}"
  fi
}

# Create directories
create_directories() {
  if [ "$USER_INSTALL" = true ]; then
    CONFIG_DIR="$HOME/.config/esnode"
    DATA_DIR="$HOME/.local/share/esnode"
    LOG_DIR="$HOME/.local/share/esnode/logs"
  fi
  
  echo -e "${BLUE}==>${NC} Creating directories"
  mkdir -p "$CONFIG_DIR"
  mkdir -p "$DATA_DIR/tsdb"
  mkdir -p "$LOG_DIR"
  
  if [ "$USER_INSTALL" = false ] && [ "$OS" = "linux" ]; then
    chown -R "$SERVICE_USER:$SERVICE_USER" "$DATA_DIR" "$LOG_DIR"
    chmod 750 "$DATA_DIR" "$LOG_DIR"
  fi
  
  echo -e "${GREEN}✓ Directories created${NC}"
}

# Generate default configuration
generate_config() {
  CONFIG_FILE="$CONFIG_DIR/config.toml"
  
  if [ -f "$CONFIG_FILE" ]; then
    echo -e "${YELLOW}⚠ Configuration already exists: $CONFIG_FILE${NC}"
    return 0
  fi
  
  echo -e "${BLUE}==>${NC} Generating default configuration"
  
  cat > "$CONFIG_FILE" << 'CONFIGEOF'
# ESNODE-Core Configuration
# Version: 0.2.0

# HTTP listener address for metrics endpoint
listen_address = "0.0.0.0:9100"

# Scrape interval (e.g. "5s", "10s", "1m")
scrape_interval = "5s"

# Collector toggles
enable_cpu = true
enable_memory = true
enable_disk = true
enable_network = true
enable_gpu = true
enable_power = true

# GPU configuration
enable_gpu_amd = false
enable_gpu_mig = false
enable_gpu_events = false
# gpu_visible_devices = ""  # Empty = all GPUs
# mig_config_devices = ""

# Kubernetes mode (emits compat metrics)
k8s_mode = false

# Power monitoring
# node_power_envelope_watts = 1500

# Local TSDB
enable_local_tsdb = true
local_tsdb_path = "/var/lib/esnode/tsdb"
local_tsdb_retention_hours = 48
local_tsdb_max_disk_mb = 2048

# Application metrics
enable_app = false
# app_metrics_url = "http://127.0.0.1:8000/metrics"

# Orchestrator (Autonomous mode)
enable_orchestrator = false

# Logging
log_level = "info"  # error, warn, info, debug, trace
CONFIGEOF

  if [ "$USER_INSTALL" = true ]; then
    sed -i.bak "s|/var/lib/esnode/tsdb|$DATA_DIR/tsdb|g" "$CONFIG_FILE"
    rm -f "$CONFIG_FILE.bak"
  elif [ "$OS" = "linux" ]; then
    chown "$SERVICE_USER:$SERVICE_USER" "$CONFIG_FILE"
  fi
  
  echo -e "${GREEN}✓ Configuration generated: $CONFIG_FILE${NC}"
}

# Install systemd service (Linux only, system install)
install_systemd_service() {
  if [ "$USER_INSTALL" = true ] || [ "$OS" != "linux" ]; then
    return 0
  fi
  
  SERVICE_FILE="$SERVICE_DIR/esnode-core.service"
  
  echo -e "${BLUE}==>${NC} Installing systemd service"
  
  cat > "$SERVICE_FILE" << SERVICEEOF
[Unit]
Description=ESNODE-Core Agent - Power-Aware AI Infrastructure
Documentation=https://github.com/ESNODE/ESNODE-Core
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_USER
ExecStart=$INSTALL_DIR/esnode-core daemon --config $CONFIG_DIR/config.toml
Restart=on-failure
RestartSec=5s
StandardOutput=journal
StandardError=journal
SyslogIdentifier=esnode-core

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$DATA_DIR $LOG_DIR
CapabilityBoundingSet=

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
SERVICEEOF

  systemctl daemon-reload
  echo -e "${GREEN}✓ Systemd service installed${NC}"
  echo -e "${CYAN}  Start with: sudo systemctl start esnode-core${NC}"
  echo -e "${CYAN}  Enable at boot: sudo systemctl enable esnode-core${NC}"
}

# Install launchd service (macOS only)
install_launchd_service() {
  if [ "$OS" != "darwin" ]; then
    return 0
  fi
  
  if [ "$USER_INSTALL" = true ]; then
    LAUNCHD_DIR="$HOME/Library/LaunchAgents"
    LABEL="com.estimatedstocks.esnode-core"
  else
    LAUNCHD_DIR="/Library/LaunchDaemons"
    LABEL="com.estimatedstocks.esnode-core"
  fi
  
  mkdir -p "$LAUNCHD_DIR"
  PLIST_FILE="$LAUNCHD_DIR/$LABEL.plist"
  
  echo -e "${BLUE}==>${NC} Installing launchd service"
  
  cat > "$PLIST_FILE" << PLISTEOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/esnode-core</string>
        <string>daemon</string>
        <string>--config</string>
        <string>$CONFIG_DIR/config.toml</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$LOG_DIR/esnode-core.log</string>
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/esnode-core.error.log</string>
</dict>
</plist>
PLISTEOF

  echo -e "${GREEN}✓ LaunchD service installed${NC}"
  echo -e "${CYAN}  Start with: launchctl load $PLIST_FILE${NC}"
}

# Print summary
print_summary() {
  echo ""
  echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${GREEN}  ✓ ESNODE-Core Installation Complete${NC}"
  echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo ""
  echo -e "${CYAN}Binary:${NC}       $INSTALL_DIR/esnode-core"
  echo -e "${CYAN}Config:${NC}       $CONFIG_DIR/config.toml"
  echo -e "${CYAN}Data:${NC}         $DATA_DIR"
  echo -e "${CYAN}Logs:${NC}         $LOG_DIR"
  echo ""
  echo -e "${YELLOW}Quick Start:${NC}"
  echo ""
  
  if [ "$USER_INSTALL" = true ]; then
    echo -e "  1. Start daemon:"
    echo -e "     ${BLUE}$INSTALL_DIR/esnode-core daemon${NC}"
    echo ""
    echo -e "  2. View metrics:"
    echo -e "     ${BLUE}curl http://localhost:9100/metrics${NC}"
    echo ""
    echo -e "  3. Launch TUI:"
    echo -e "     ${BLUE}$INSTALL_DIR/esnode-core cli${NC}"
  else
    if [ "$OS" = "linux" ]; then
      echo -e "  1. Start service:"
      echo -e "     ${BLUE}sudo systemctl start esnode-core${NC}"
      echo ""
      echo -e "  2. Enable at boot:"
      echo -e "     ${BLUE}sudo systemctl enable esnode-core${NC}"
      echo ""
      echo -e "  3. View status:"
      echo -e "     ${BLUE}sudo systemctl status esnode-core${NC}"
      echo ""
      echo -e "  4. View logs:"
      echo -e "     ${BLUE}sudo journalctl -u esnode-core -f${NC}"
      echo ""
      echo -e "  5. Launch TUI:"
      echo -e "     ${BLUE}esnode-core cli${NC}"
    elif [ "$OS" = "darwin" ]; then
      echo -e "  1. Start service:"
      echo -e "     ${BLUE}sudo launchctl load $LAUNCHD_DIR/$LABEL.plist${NC}"
      echo ""
      echo -e "  2. Launch TUI:"
      echo -e "     ${BLUE}esnode-core cli${NC}"
    fi
  fi
  
  echo ""
  echo -e "${YELLOW}Documentation:${NC}"
  echo -e "  ${BLUE}https://github.com/ESNODE/ESNODE-Core${NC}"
  echo ""
  echo -e "${YELLOW}Endpoints:${NC}"
  echo -e "  Metrics:  ${BLUE}http://localhost:9100/metrics${NC}"
  echo -e "  Health:   ${BLUE}http://localhost:9100/health${NC}"
  echo -e "  Status:   ${BLUE}http://localhost:9100/status${NC}"
  echo ""
}

# Main installation flow
main() {
  echo -e "${BLUE}==>${NC} Starting installation..."
  echo ""
  
  detect_platform
  check_binary
  install_binary
  create_service_user
  create_directories
  generate_config
  install_systemd_service
  install_launchd_service
  print_summary
}

main "$@"
