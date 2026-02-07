#!/usr/bin/env bash
# ESNODE-Core Release Build Script
# Version: 0.2.0
# Builds release binaries for multiple platforms

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT_DIR/public/distribution"
VERSION_FILE="$ROOT_DIR/crates/agent-bin/Cargo.toml"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Banner
echo -e "${CYAN}"
cat << "EOF"
  ✱ ESNODE
  Power-Aware AI Infrastructure
  Release Build Script
EOF
echo -e "${NC}"

# Read version
AGENT_VER=$(grep '^version = ' "$VERSION_FILE" | head -n1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${GREEN}Version: v${AGENT_VER}${NC}"
echo ""

# Create output directory
mkdir -p "$OUT_DIR"

# Detect host platform
HOST_OS=$(uname -s | tr '[:upper:]' '[:lower:]')
HOST_ARCH=$(uname -m)

case "$HOST_ARCH" in
  x86_64) HOST_ARCH="amd64" ;;
  aarch64|arm64) HOST_ARCH="arm64" ;;
esac

echo -e "${BLUE}==>${NC} Host platform: $HOST_OS-$HOST_ARCH"
echo ""

# Build release binary for host
build_host() {
  echo -e "${BLUE}==>${NC} Building release binary for $HOST_OS-$HOST_ARCH"
  cargo build --release -p agent-bin
  
  BIN_PATH="$ROOT_DIR/target/release/esnode-core"
  if [ ! -f "$BIN_PATH" ]; then
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
  fi
  
  # Copy to dist
  cp "$BIN_PATH" "$OUT_DIR/esnode-core-$HOST_OS-$HOST_ARCH"
  
  echo -e "${GREEN}✓ Built: esnode-core-$HOST_OS-$HOST_ARCH${NC}"
}

# Create tarball with install script
create_tarball() {
  local os=$1
  local arch=$2
  local binary="esnode-core-${os}-${arch}"
  
  if [ ! -f "$OUT_DIR/$binary" ]; then
    echo -e "${YELLOW}⚠ Skipping tarball for $os-$arch (binary not found)${NC}"
    return 0
  fi
  
  echo -e "${BLUE}==>${NC} Packaging $os-$arch"
  
  local pkg_dir="$OUT_DIR/esnode-core-${os}-${arch}-v${AGENT_VER}"
  mkdir -p "$pkg_dir"
  
  # Copy binary
  cp "$OUT_DIR/$binary" "$pkg_dir/esnode-core"
  chmod +x "$pkg_dir/esnode-core"
  
  # Copy install script
  cp "$ROOT_DIR/scripts/install.sh" "$pkg_dir/install.sh"
  chmod +x "$pkg_dir/install.sh"
  
  # Create README
  cat > "$pkg_dir/README.txt" << 'README'
ESNODE-Core v${AGENT_VER}
Power-Aware AI Infrastructure

INSTALLATION
============

Quick Install (requires root):
  sudo ./install.sh

User Install (no root):
  INSTALL_DIR=$HOME/.local/bin ./install.sh

Manual Install:
  1. Copy esnode-core to /usr/local/bin/ (or ~/.local/bin/)
  2. Make executable: chmod +x /usr/local/bin/esnode-core
  3. Run: esnode-core daemon

USAGE
=====

Start daemon:
  esnode-core daemon

Launch TUI:
  esnode-core cli

View status:
  esnode-core status

Check metrics:
  curl http://localhost:9100/metrics

DOCUMENTATION
=============

GitHub: https://github.com/ESNODE/ESNODE-Core
License: BUSL-1.1 (Source Available)

Copyright (c) 2024 Estimatedstocks AB
README
  
  sed -i.bak "s/\${AGENT_VER}/$AGENT_VER/g" "$pkg_dir/README.txt"
  rm -f "$pkg_dir/README.txt.bak"
  
  # Create tarball
  tar -C "$OUT_DIR" -czf "$OUT_DIR/esnode-core-${os}-${arch}-v${AGENT_VER}.tar.gz" \
    "esnode-core-${os}-${arch}-v${AGENT_VER}"
  
  # Cleanup temp directory
  rm -rf "$pkg_dir"
  
  echo -e "${GREEN}✓ Created: esnode-core-${os}-${arch}-v${AGENT_VER}.tar.gz${NC}"
}

# Generate checksums
generate_checksums() {
  echo -e "${BLUE}==>${NC} Generating checksums"
  
  cd "$OUT_DIR"
  shasum -a 256 *.tar.gz > "SHA256SUMS-v${AGENT_VER}.txt" 2>/dev/null || true
  
  if [ -f "SHA256SUMS-v${AGENT_VER}.txt" ]; then
    echo -e "${GREEN}✓ Created: SHA256SUMS-v${AGENT_VER}.txt${NC}"
  fi
  
  cd "$ROOT_DIR"
}

# Cross-compile for Linux (if possible)
cross_compile_linux() {
  if [ "$HOST_OS" = "darwin" ] && command -v cross &>/dev/null; then
    echo -e "${BLUE}==>${NC} Cross-compiling for Linux (using cross)"
    
    # Linux x86_64
    if cross build --release --target x86_64-unknown-linux-gnu -p agent-bin 2>/dev/null; then
      BIN="$ROOT_DIR/target/x86_64-unknown-linux-gnu/release/esnode-core"
      if [ -f "$BIN" ]; then
        cp "$BIN" "$OUT_DIR/esnode-core-linux-amd64"
        echo -e "${GREEN}✓ Built: esnode-core-linux-amd64${NC}"
      fi
    fi
    
    # Linux ARM64
    if cross build --release --target aarch64-unknown-linux-gnu -p agent-bin 2>/dev/null; then
      BIN="$ROOT_DIR/target/aarch64-unknown-linux-gnu/release/esnode-core"
      if [ -f "$BIN" ]; then
        cp "$BIN" "$OUT_DIR/esnode-core-linux-arm64"
        echo -e "${GREEN}✓ Built: esnode-core-linux-arm64${NC}"
      fi
    fi
  fi
}

# Create release notes
create_release_notes() {
  echo -e "${BLUE}==>${NC} Generating release notes"
  
  cat > "$OUT_DIR/RELEASE_NOTES-v${AGENT_VER}.md" << 'NOTES'
# ESNODE-Core v${AGENT_VER}

**Release Date:** $(date +%Y-%m-%d)

## What's New

### Brand Identity Update
- New ESNODE logo with asterisk symbol (✱)
- Updated tagline: "Power-Aware AI Infrastructure"
- Professional dark navy color scheme

### TUI Modernization
- Complete dashboard redesign matching cloud-provider quality
- Enhanced sidebar navigation with Material Design colors
- Professional gauges, tables, and status indicators
- 7 fully redesigned screens (Overview, GPU, Network, Efficiency, Orchestrator, Metrics, Status)

### Documentation
- Comprehensive brand guidelines
- Complete TUI user guide
- Enhanced README with logo and badges

## Installation

### Linux
```bash
# Download and extract
tar -xzf esnode-core-linux-amd64-v${AGENT_VER}.tar.gz
cd esnode-core-linux-amd64-v${AGENT_VER}

# Install (requires root)
sudo ./install.sh

# Or manual install
sudo cp esnode-core /usr/local/bin/
sudo chmod +x /usr/local/bin/esnode-core
```

### macOS
```bash
# Download and extract
tar -xzf esnode-core-darwin-arm64-v${AGENT_VER}.tar.gz
cd esnode-core-darwin-arm64-v${AGENT_VER}

# Install
sudo ./install.sh

# Or use Homebrew (if available)
brew install esnode-core
```

## Quick Start

```bash
# Start daemon
esnode-core daemon

# Launch TUI
esnode-core cli

# View metrics
curl http://localhost:9100/metrics
```

## Verification

Verify checksums:
```bash
shasum -a 256 -c SHA256SUMS-v${AGENT_VER}.txt
```

## Documentation

- **GitHub:** https://github.com/ESNODE/ESNODE-Core
- **Brand Guidelines:** docs/BRAND_GUIDELINES.md
- **TUI User Guide:** docs/TUI_USER_GUIDE.md

## License

BUSL-1.1 (Source Available)
Copyright (c) 2024 Estimatedstocks AB

## Support

Report issues: https://github.com/ESNODE/ESNODE-Core/issues
NOTES

  sed -i.bak "s/\${AGENT_VER}/$AGENT_VER/g" "$OUT_DIR/RELEASE_NOTES-v${AGENT_VER}.md"
  sed -i.bak "s/\$(date +%Y-%m-%d)/$(date +%Y-%m-%d)/g" "$OUT_DIR/RELEASE_NOTES-v${AGENT_VER}.md"
  rm -f "$OUT_DIR/RELEASE_NOTES-v${AGENT_VER}.md.bak"
  
  echo -e "${GREEN}✓ Created: RELEASE_NOTES-v${AGENT_VER}.md${NC}"
}

# Summary
print_summary() {
  echo ""
  echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${GREEN}  ✓ Release Build Complete${NC}"
  echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo ""
  echo -e "${CYAN}Output directory:${NC} $OUT_DIR"
  echo ""
  echo -e "${CYAN}Artifacts:${NC}"
  ls -lh "$OUT_DIR"/*.tar.gz 2>/dev/null | awk '{printf "  %s (%s)\n", $9, $5}' || true
  echo ""
  echo -e "${YELLOW}Next steps:${NC}"
  echo -e "  1. Verify checksums"
  echo -e "  2. Test installation on target platforms"
  echo -e "  3. Create GitHub release"
  echo -e "  4. Upload artifacts"
  echo ""
}

# Main
main() {
  build_host
  cross_compile_linux
  
  # Create tarballs for all binaries
  create_tarball "$HOST_OS" "$HOST_ARCH"
  create_tarball "linux" "amd64"
  create_tarball "linux" "arm64"
  
  generate_checksums
  create_release_notes
  print_summary
}

main "$@"
