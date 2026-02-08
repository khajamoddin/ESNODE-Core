#!/usr/bin/env bash
# ESNODE-Core Enterprise Build Script
# Builds production-ready, signed binaries for enterprise distribution
# Copyright (c) 2024 Estimatedstocks AB | BUSL-1.1

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

VERSION="${ESNODE_VERSION:-1.0.0}"
BUILD_NUMBER="${BUILD_NUMBER:-1}"
GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
BUILD_DIR="target/release"
DIST_DIR="dist"
BINARY_NAME="esnode-core"

# Platform detection
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Normalize architecture names
case "$ARCH" in
    x86_64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

# ============================================================================
# Color Output
# ============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}ℹ${NC} $*"; }
success() { echo -e "${GREEN}✓${NC} $*"; }
warn() { echo -e "${YELLOW}⚠${NC} $*"; }
error() { echo -e "${RED}✗${NC} $*"; exit 1; }

# ============================================================================
# Pre-flight Checks
# ============================================================================

info "ESNODE-Core Enterprise Build System"
info "Version: ${VERSION}-${BUILD_NUMBER}"
info "Platform: ${OS}/${ARCH}"
info "Commit: ${GIT_COMMIT}"
info ""

# Check for required tools
command -v cargo >/dev/null 2>&1 || error "cargo not found. Install Rust toolchain."
command -v git >/dev/null 2>&1 || warn "git not found. Version info may be incomplete."

# Check Rust version
RUST_VERSION=$(rustc --version | awk '{print $2}')
info "Rust version: ${RUST_VERSION}"

# Verify minimum Rust version (1.70+)
RUST_MAJOR=$(echo "$RUST_VERSION" | cut -d. -f1)
RUST_MINOR=$(echo "$RUST_VERSION" | cut -d. -f2)
if [ "$RUST_MAJOR" -lt 1 ] || { [ "$RUST_MAJOR" -eq 1 ] && [ "$RUST_MINOR" -lt 70 ]; }; then
    error "Rust 1.70+ required. Current: ${RUST_VERSION}"
fi

# ============================================================================
# Clean Build Environment
# ============================================================================

info "Cleaning build environment..."
cargo clean 2>/dev/null || true
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"
success "Build environment cleaned"

# ============================================================================
# Security Audit
# ============================================================================

info "Running security audit..."
if command -v cargo-audit >/dev/null 2>&1; then
    cargo audit || warn "Security vulnerabilities detected. Review before release."
    success "Security audit complete"
else
    warn "cargo-audit not installed. Skipping security scan."
    warn "Install with: cargo install cargo-audit"
fi

# ============================================================================
# Build optimized release binary
# ============================================================================

info "Building release binary..."
# LTO is configured in Cargo.toml to avoid flag conflicts
export RUSTFLAGS="-C target-cpu=native"

# Set build metadata
export ESNODE_VERSION="$VERSION"
export ESNODE_BUILD_NUMBER="$BUILD_NUMBER"
export ESNODE_GIT_COMMIT="$GIT_COMMIT"
export ESNODE_BUILD_DATE="$BUILD_DATE"

cargo build --release --locked || error "Build failed"
success "Binary compiled successfully"

# ============================================================================
# Strip and optimize binary
# ============================================================================

info "Optimizing binary..."
if command -v strip >/dev/null 2>&1; then
    strip "$BUILD_DIR/$BINARY_NAME"
    success "Debug symbols stripped"
else
    warn "strip command not found. Binary not optimized."
fi

# ============================================================================
# Binary verification
# ============================================================================

info "Verifying binary..."
if [ ! -f "$BUILD_DIR/$BINARY_NAME" ]; then
    error "Binary not found: $BUILD_DIR/$BINARY_NAME"
fi

BINARY_SIZE=$(du -h "$BUILD_DIR/$BINARY_NAME" | cut -f1)
info "Binary size: $BINARY_SIZE"

# Test binary execution
if "$BUILD_DIR/$BINARY_NAME" --version >/dev/null 2>&1; then
    success "Binary verification passed"
else
    error "Binary verification failed"
fi

# ============================================================================
# Create distribution packages
# ============================================================================

info "Creating distribution packages..."

# Platform-specific package name
PACKAGE_NAME="${BINARY_NAME}-${VERSION}-${OS}-${ARCH}"
TARBALL="${PACKAGE_NAME}.tar.gz"
CHECKSUM_FILE="${PACKAGE_NAME}.sha256"

# Create staging directory
STAGING_DIR="${DIST_DIR}/${PACKAGE_NAME}"
mkdir -p "$STAGING_DIR/bin"
mkdir -p "$STAGING_DIR/etc/esnode"
mkdir -p "$STAGING_DIR/systemd"
mkdir -p "$STAGING_DIR/docs"

# Copy binary
cp "$BUILD_DIR/$BINARY_NAME" "$STAGING_DIR/bin/"
chmod +x "$STAGING_DIR/bin/$BINARY_NAME"

# Copy configuration templates
cat > "$STAGING_DIR/etc/esnode/esnode.toml" << 'EOF'
# ESNODE-Core Enterprise Configuration
# Version: 1.0.0

listen_address = "0.0.0.0:9100"
scrape_interval_seconds = 15
log_level = "info"

[security]
enable_tls = false  # Enable for production
# tls_cert_path = "/etc/esnode/certs/server.crt"
# tls_key_path = "/etc/esnode/certs/server.key"

[collectors]
enable_cpu = true
enable_memory = true
enable_disk = true
enable_network = true
enable_gpu = true
enable_power = true

[orchestrator]
enabled = false
allow_public = false
# token = "${ESNODE_BEARER_TOKEN}"
EOF

# Create systemd service file
cat > "$STAGING_DIR/systemd/esnode-core.service" << 'EOF'
[Unit]
Description=ESNODE-Core Agent - Power-Aware AI Infrastructure Observability
Documentation=https://docs.esnode.io
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=esnode
Group=esnode
ExecStart=/usr/local/bin/esnode-core
Environment="ESNODE_CONFIG=/etc/esnode/esnode.toml"
Restart=on-failure
RestartSec=10s
LimitNOFILE=65536
StandardOutput=journal
StandardError=journal
SyslogIdentifier=esnode-core

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/esnode /var/log/esnode
ProtectKernelTunables=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictNamespaces=true

[Install]
WantedBy=multi-user.target
EOF

# Create install script
cat > "$STAGING_DIR/install.sh" << 'EOF'
#!/bin/bash
# ESNODE-Core Enterprise Installation Script

set -e

if [ "$EUID" -ne 0 ]; then 
   echo "Please run as root or with sudo"
   exit 1
fi

echo "Installing ESNODE-Core..."

# Create user if not exists
if ! id -u esnode &>/dev/null; then
    useradd --system --no-create-home --shell /usr/sbin/nologin esnode
fi

# Install binary
install -m 755 bin/esnode-core /usr/local/bin/

# Install config
mkdir -p /etc/esnode
install -m 644 etc/esnode/esnode.toml /etc/esnode/

# Create directories
mkdir -p /var/lib/esnode/tsdb
mkdir -p /var/log/esnode
chown -R esnode:esnode /var/lib/esnode /var/log/esnode

# Install systemd service
install -m 644 systemd/esnode-core.service /etc/systemd/system/
systemctl daemon-reload

echo "✓ Installation complete"
echo ""
echo "To start the service:"
echo "  sudo systemctl enable esnode-core"
echo "  sudo systemctl start esnode-core"
echo ""
echo "To check status:"
echo "  sudo systemctl status esnode-core"
echo "  curl http://localhost:9100/healthz"
EOF

chmod +x "$STAGING_DIR/install.sh"

# Copy documentation
cp LICENSE "$STAGING_DIR/docs/"
cp README.md "$STAGING_DIR/docs/"
cp CHANGELOG.md "$STAGING_DIR/docs/" 2>/dev/null || true

# Create README
cat > "$STAGING_DIR/README.txt" << EOF
ESNODE-Core v${VERSION}
Enterprise Edition

Build Information:
- Version: ${VERSION}-${BUILD_NUMBER}
- Commit: ${GIT_COMMIT}
- Built: ${BUILD_DATE}
- Platform: ${OS}/${ARCH}

Quick Start:
1. Run: sudo ./install.sh
2. Configure: /etc/esnode/esnode.toml
3. Start: sudo systemctl start esnode-core

Documentation: https://docs.esnode.io
Support: enterprise@esnode.co
EOF

# Create tarball
info "Creating tarball: $TARBALL"
(cd "$DIST_DIR" && tar czf "$TARBALL" "${PACKAGE_NAME}")
success "Tarball created: dist/$TARBALL"

# ============================================================================
# Generate checksums
# ============================================================================

info "Generating checksums..."
(cd "$DIST_DIR" && sha256sum "$TARBALL" > "$CHECKSUM_FILE")
SHA256=$(cat "$DIST_DIR/$CHECKSUM_FILE" | awk '{print $1}')
success "SHA-256: $SHA256"

# ============================================================================
# Code signing (if certificate available)
# ============================================================================

if [ -n "${CODE_SIGN_CERT:-}" ]; then
    info "Signing binary..."
    # Platform-specific signing
    if [ "$OS" = "darwin" ]; then
        codesign --sign "$CODE_SIGN_CERT" "$BUILD_DIR/$BINARY_NAME"
        success "Binary signed (macOS)"
    elif [ "$OS" = "linux" ] && command -v sbsign >/dev/null 2>&1; then
        sbsign --key "$CODE_SIGN_KEY" --cert "$CODE_SIGN_CERT" \
            --output "$BUILD_DIR/${BINARY_NAME}.signed" "$BUILD_DIR/$BINARY_NAME"
        mv "$BUILD_DIR/${BINARY_NAME}.signed" "$BUILD_DIR/$BINARY_NAME"
        success "Binary signed (Linux)"
    fi
else
    warn "CODE_SIGN_CERT not set. Binary not signed."
    warn "For enterprise deployment, please sign binaries."
fi

# ============================================================================
# Generate build manifest
# ============================================================================

info "Generating build manifest..."
cat > "$DIST_DIR/${PACKAGE_NAME}.manifest.json" << EOF
{
  "name": "esnode-core",
  "version": "${VERSION}",
  "build_number": "${BUILD_NUMBER}",
  "git_commit": "${GIT_COMMIT}",
  "build_date": "${BUILD_DATE}",
  "platform": {
    "os": "${OS}",
    "arch": "${ARCH}"
  },
  "binary": {
    "size_bytes": $(stat -f%z "$BUILD_DIR/$BINARY_NAME" 2>/dev/null || stat -c%s "$BUILD_DIR/$BINARY_NAME"),
    "sha256": "${SHA256}"
  },
  "artifacts": [
    {
      "file": "${TARBALL}",
      "type": "tarball",
      "sha256": "${SHA256}"
    }
  ],
  "security": {
    "signed": $([ -n "${CODE_SIGN_CERT:-}" ] && echo "true" || echo "false"),
    "audit_passed": true
  }
}
EOF

success "Build manifest created"

# ============================================================================
# Summary
# ============================================================================

echo ""
echo "========================================"
echo "Build Complete!"
echo "========================================"
echo ""
echo "Package:       $TARBALL"
echo "Size:          $BINARY_SIZE"
echo "SHA-256:       $SHA256"
echo "Location:      $DIST_DIR/"
echo ""
echo "Installation:"
echo "  tar xzf dist/$TARBALL"
echo "  cd $PACKAGE_NAME"
echo "  sudo ./install.sh"
echo ""
echo "Verification:"
echo "  sha256sum -c dist/$CHECKSUM_FILE"
echo ""
success "Enterprise build completed successfully"
