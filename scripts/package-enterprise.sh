#!/usr/bin/env bash
# ESNODE-Core Enterprise Package Builder
# Creates RPM, DEB, and container images for enterprise distribution
# Copyright (c) 2024 Estimatedstocks AB | BUSL-1.1

set -euo pipefail

VERSION="${ESNODE_VERSION:-1.0.0}"
BUILD_NUMBER="${BUILD_NUMBER:-1}"
RELEASE="1"

# ============================================================================
# Build RPM Package (RHEL/CentOS/Rocky/AlmaLinux)
# ============================================================================

build_rpm() {
    echo "Building RPM package..."
    
    if ! command -v rpmbuild &>/dev/null; then
        echo "rpmbuild not found. Install: sudo yum install rpm-build"
        return 1
    fi
    
    # Create RPM build structure
    mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
    
    # Create spec file
    cat > ~/rpmbuild/SPECS/esnode-core.spec << EOF
Name:           esnode-core
Version:        ${VERSION}
Release:        ${RELEASE}%{?dist}
Summary:        Power-Aware AI Infrastructure Observability Platform
License:        BUSL-1.1
URL:            https://esnode.io
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  systemd
Requires:       systemd
Requires(pre):  shadow-utils

%description
ESNODE-Core is an enterprise-grade observability platform for AI infrastructure.
It provides real-time telemetry, autonomous operations, and predictive maintenance
for GPU-accelerated compute clusters.

%prep
%setup -q

%build
# Binary pre-compiled in release workflow

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_sysconfdir}/esnode
mkdir -p %{buildroot}%{_unitdir}
mkdir -p %{buildroot}%{_localstatedir}/lib/esnode
mkdir -p %{buildroot}%{_localstatedir}/log/esnode

install -m 755 bin/esnode-core %{buildroot}%{_bindir}/
install -m 644 etc/esnode/esnode.toml %{buildroot}%{_sysconfdir}/esnode/
install -m 644 systemd/esnode-core.service %{buildroot}%{_unitdir}/

%pre
getent group esnode >/dev/null || groupadd -r esnode
getent passwd esnode >/dev/null || \
    useradd -r -g esnode -d /var/lib/esnode -s /sbin/nologin \
    -c "ESNODE Core Agent" esnode
exit 0

%post
%systemd_post esnode-core.service

%preun
%systemd_preun esnode-core.service

%postun
%systemd_postun_with_restart esnode-core.service

%files
%license docs/LICENSE
%doc docs/README.md docs/CHANGELOG.md
%{_bindir}/esnode-core
%config(noreplace) %{_sysconfdir}/esnode/esnode.toml
%{_unitdir}/esnode-core.service
%attr(0755,esnode,esnode) %dir %{_localstatedir}/lib/esnode
%attr(0755,esnode,esnode) %dir %{_localstatedir}/log/esnode

%changelog
* $(date +'%a %b %d %Y') Package Maintainer <engineering@esnode.co> - ${VERSION}-${RELEASE}
- Release ${VERSION}
EOF
    
    # Build RPM
    rpmbuild -ba ~/rpmbuild/SPECS/esnode-core.spec
    
    echo "✓ RPM package built: ~/rpmbuild/RPMS/x86_64/esnode-core-${VERSION}-${RELEASE}.*.rpm"
}

# ============================================================================
# Build DEB Package (Ubuntu/Debian)
# ============================================================================

build_deb() {
    echo "Building DEB package..."
    
    if ! command -v dpkg-deb &>/dev/null; then
        echo "dpkg-deb not found. Install: sudo apt-get install dpkg-dev"
        return 1
    fi
    
    DEB_DIR="esnode-core_${VERSION}"
    mkdir -p "$DEB_DIR/DEBIAN"
    mkdir -p "$DEB_DIR/usr/local/bin"
    mkdir -p "$DEB_DIR/etc/esnode"
    mkdir -p "$DEB_DIR/lib/systemd/system"
    mkdir -p "$DEB_DIR/var/lib/esnode"
    mkdir -p "$DEB_DIR/var/log/esnode"
    
    # Control file
    cat > "$DEB_DIR/DEBIAN/control" << EOF
Package: esnode-core
Version: ${VERSION}-${BUILD_NUMBER}
Section: admin
Priority: optional
Architecture: amd64
Depends: systemd, adduser
Maintainer: ESNODE Team <engineering@esnode.co>
Homepage: https://esnode.io
Description: Power-Aware AI Infrastructure Observability
 ESNODE-Core is an enterprise-grade observability platform for AI infrastructure.
 It provides real-time telemetry, autonomous operations, and predictive maintenance
 for GPU-accelerated compute clusters.
 .
 Key Features:
  - GPU telemetry (NVIDIA NVML)
  - CPU, memory, disk, network monitoring
  - Autonomous root cause analysis (RCA)
  - Predictive maintenance (ML-based failure prediction)
  - Prometheus-native metrics export
  - Efficiency as Code (declarative policy engine)
EOF
    
    # Pre-install script
    cat > "$DEB_DIR/DEBIAN/preinst" << 'EOF'
#!/bin/bash
set -e
if ! getent group esnode >/dev/null; then
    addgroup --system esnode
fi
if ! getent passwd esnode >/dev/null; then
    adduser --system --ingroup esnode --home /var/lib/esnode \
        --no-create-home --shell /usr/sbin/nologin esnode
fi
EOF
    chmod 755 "$DEB_DIR/DEBIAN/preinst"
    
    # Post-install script
    cat > "$DEB_DIR/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e
chown -R esnode:esnode /var/lib/esnode /var/log/esnode
systemctl daemon-reload
systemctl enable esnode-core.service || true
echo "ESNODE-Core installed successfully"
echo "Start with: sudo systemctl start esnode-core"
EOF
    chmod 755 "$DEB_DIR/DEBIAN/postinst"
    
    # Pre-remove script
    cat > "$DEB_DIR/DEBIAN/prerm" << 'EOF'
#!/bin/bash
set -e
systemctl stop esnode-core.service || true
systemctl disable esnode-core.service || true
EOF
    chmod 755 "$DEB_DIR/DEBIAN/prerm"
    
    # Copy files
    cp target/release/esnode-core "$DEB_DIR/usr/local/bin/"
    cp etc/esnode/esnode.toml "$DEB_DIR/etc/esnode/"
    cp systemd/esnode-core.service "$DEB_DIR/lib/systemd/system/"
    
    # Build package
    dpkg-deb --build "$DEB_DIR"
    
    echo "✓ DEB package built: esnode-core_${VERSION}.deb"
}

# ============================================================================
# Build Container Image
# ============================================================================

build_container() {
    echo "Building container image..."
    
    if ! command -v docker &>/dev/null; then
        echo "docker not found. Install Docker to build containers."
        return 1
    fi
    
    # Create optimized Dockerfile
    cat > Dockerfile.enterprise << 'EOF'
# ESNODE-Core Enterprise Container
# Multi-stage build for minimal attack surface

# Build stage (not used - binary pre-compiled)
FROM rust:1.75-alpine AS builder
WORKDIR /build

# Runtime stage
FROM alpine:3.19
LABEL maintainer="ESNODE Team <engineering@esnode.co>"
LABEL org.opencontainers.image.title="ESNODE-Core"
LABEL org.opencontainers.image.description="Power-Aware AI Infrastructure Observability"
LABEL org.opencontainers.image.vendor="Estimatedstocks AB"
LABEL org.opencontainers.image.licenses="BUSL-1.1"
LABEL org.opencontainers.image.documentation="https://docs.esnode.io"

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc tini

# Create non-root user
RUN addgroup -S esnode && adduser -S -G esnode -h /var/lib/esnode esnode

# Copy binary
COPY target/release/esnode-core /usr/local/bin/esnode-core
RUN chmod +x /usr/local/bin/esnode-core

# Create directories
RUN mkdir -p /etc/esnode /var/lib/esnode/tsdb /var/log/esnode && \
    chown -R esnode:esnode /var/lib/esnode /var/log/esnode

# Copy default config
COPY etc/esnode/esnode.toml /etc/esnode/esnode.toml

# Switch to non-root user
USER esnode
WORKDIR /var/lib/esnode

# Expose metrics port
EXPOSE 9100

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:9100/healthz || exit 1

# Use tini for proper signal handling
ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/usr/local/bin/esnode-core"]
EOF
    
    # Build image
    docker build -f Dockerfile.enterprise \
        -t esnode/esnode-core:${VERSION} \
        -t esnode/esnode-core:latest \
        .
    
    # Scan for vulnerabilities (if trivy installed)
    if command -v trivy &>/dev/null; then
        trivy image esnode/esnode-core:${VERSION}
    fi
    
    echo "✓ Container image built: esnode/esnode-core:${VERSION}"
}

# ============================================================================
# Build Helm Chart
# ============================================================================

build_helm_chart() {
    echo "Building Helm chart..."
    
    CHART_DIR="helm/esnode-core"
    mkdir -p "$CHART_DIR/templates"
    
    # Chart.yaml
    cat > "$CHART_DIR/Chart.yaml" << EOF
apiVersion: v2
name: esnode-core
description: Power-Aware AI Infrastructure Observability Platform
type: application
version: ${VERSION}
appVersion: "${VERSION}"
keywords:
  - observability
  - monitoring
  - gpu
  - ai
  - ml
home: https://esnode.io
sources:
  - https://github.com/estimatedstocks/esnode-core
maintainers:
  - name: ESNODE Team
    email: engineering@esnode.co
EOF
    
    # values.yaml
    cat > "$CHART_DIR/values.yaml" << EOF
# ESNODE-Core Helm Chart Values
# Enterprise-grade defaults for production deployments

image:
  repository: esnode/esnode-core
  pullPolicy: IfNotPresent
  tag: "${VERSION}"

imagePullSecrets: []

serviceAccount:
  create: true
  annotations: {}
  name: "esnode-core"

podAnnotations:
  prometheus.io/scrape: "true"
  prometheus.io/port: "9100"
  prometheus.io/path: "/metrics"

podSecurityContext:
  runAsNonRoot: true
  runAsUser: 1000
  fsGroup: 1000
  seccompProfile:
    type: RuntimeDefault

securityContext:
  capabilities:
    drop:
    - ALL
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false

service:
  type: ClusterIP
  port: 9100
  targetPort: 9100

resources:
  limits:
    cpu: 500m
    memory: 512Mi
  requests:
    cpu: 100m
    memory: 128Mi

nodeSelector: {}

tolerations:
  - effect: NoSchedule
    key: nvidia.com/gpu
    operator: Exists

affinity: {}

config:
  scrapeInterval: 15
  logLevel: "info"
  enableTLS: false
  
orchestrator:
  enabled: false
  allowPublic: false
  tokenSecretName: esnode-token

tsdb:
  enabled: true
  retentionHours: 168
  maxDiskMB: 1024

# Security hardening
hostNetwork: false
hostPID: false
hostIPC: false
EOF
    
    # DaemonSet template
    cat > "$CHART_DIR/templates/daemonset.yaml" << 'EOF'
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: {{ include "esnode-core.fullname" . }}
  labels:
    {{- include "esnode-core.labels" . | nindent 4 }}
spec:
  selector:
    matchLabels:
      {{- include "esnode-core.selectorLabels" . | nindent 6 }}
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 10%
  template:
    metadata:
      annotations:
        {{- with .Values.podAnnotations }}
        {{- toYaml . | nindent 8 }}
        {{- end }}
      labels:
        {{- include "esnode-core.selectorLabels" . | nindent 8 }}
    spec:
      {{- with .Values.imagePullSecrets }}
      imagePullSecrets:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      serviceAccountName: {{ include "esnode-core.serviceAccountName" . }}
      securityContext:
        {{- toYaml .Values.podSecurityContext | nindent 8 }}
      hostNetwork: {{ .Values.hostNetwork }}
      hostPID: {{ .Values.hostPID }}
      hostIPC: {{ .Values.hostIPC }}
      containers:
      - name: esnode-core
        securityContext:
          {{- toYaml .Values.securityContext | nindent 12 }}
        image: "{{ .Values.image.repository }}:{{ .Values.image.tag | default .Chart.AppVersion }}"
        imagePullPolicy: {{ .Values.image.pullPolicy }}
        ports:
        - name: metrics
          containerPort: 9100
          protocol: TCP
        livenessProbe:
          httpGet:
            path: /healthz
            port: metrics
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /healthz
            port: metrics
          initialDelaySeconds: 5
          periodSeconds: 10
        resources:
          {{- toYaml .Values.resources | nindent 12 }}
        env:
        - name: ESNODE_LOG_LEVEL
          value: {{ .Values.config.logLevel | quote }}
        - name: ESNODE_SCRAPE_INTERVAL
          value: {{ .Values.config.scrapeInterval | quote }}
        volumeMounts:
        - name: tsdb
          mountPath: /var/lib/esnode/tsdb
      volumes:
      - name: tsdb
        hostPath:
          path: /var/lib/esnode/tsdb
          type: DirectoryOrCreate
      {{- with .Values.nodeSelector }}
      nodeSelector:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.tolerations }}
      tolerations:
        {{- toYaml . | nindent 8 }}
      {{- end }}
EOF
    
    # Package chart
    if command -v helm &>/dev/null; then
        helm package "$CHART_DIR"
        echo "✓ Helm chart packaged: esnode-core-${VERSION}.tgz"
    else
        echo "⚠ Helm not installed. Chart created but not packaged."
    fi
}

# ============================================================================
# Main execution
# ============================================================================

echo "ESNODE-Core Enterprise Package Builder"
echo "Version: ${VERSION}-${BUILD_NUMBER}"
echo ""

# Check if binary exists
if [ ! -f "target/release/esnode-core" ]; then
    echo "❌ Binary not found. Run build-enterprise.sh first."
    exit 1
fi

# Build packages based on platform or argument
case "${1:-all}" in
    rpm)
        build_rpm
        ;;
    deb)
        build_deb
        ;;
    container)
        build_container
        ;;
    helm)
        build_helm_chart
        ;;
    all)
        build_rpm || echo "⚠ RPM build skipped"
        build_deb || echo "⚠ DEB build skipped"
        build_container || echo "⚠ Container build skipped"
        build_helm_chart || echo "⚠ Helm chart build skipped"
        ;;
    *)
        echo "Usage: $0 {rpm|deb|container|helm|all}"
        exit 1
        ;;
esac

echo ""
echo "✓ Enterprise packaging complete"
