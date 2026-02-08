# ESNODE-Core Security Policy

**Document Version:** 1.0  
**Last Updated:** 2026-02-09  
**Owner:** Security Team  
**Classification:** Public

---

## Overview

This document outlines the security policies, practices, and vulnerability disclosure procedures for ESNODE-Core. As enterprise-grade infrastructure software, security is our highest priority.

---

## 1. Security Commitment

ESNODE-Core is designed with security-first principles:

✅ **Memory Safety:** Written in Rust to eliminate entire classes of vulnerabilities  
✅ **Principle of Least Privilege:** Minimal permissions required  
✅ **Defence in Depth:** Multiple layers of security controls  
✅ **Zero Trust Architecture:** No implicit trust in network or components  
✅ **Audit Logging:** Complete trail of all control plane operations  

---

## 2. Vulnerability Reporting

### Reporting a Security Issue

**DO NOT** create public GitHub issues for security vulnerabilities.

Instead, please report security issues to:
- **Email:** security@esnode.co
- **PGP Key:** Available at https://esnode.io/pgp-key.asc
- **Response Time:** Within 24 hours (business days)

### What to Include

Please provide:
1. Description of the vulnerability
2. Steps to reproduce
3. Affected versions
4. Potential impact assessment
5. Any proof-of-concept code (if applicable)

### Our Commitment

- **Acknowledgment:** Within 24 hours
- **Status Update:** Within 72 hours of acknowledgment
- **Fix Timeline:**
  - Critical (CVSS 9.0-10.0): 48 hours
  - High (CVSS 7.0-8.9): 7 days
  - Medium (CVSS 4.0-6.9): 30 days
  - Low (CVSS 0.1-3.9): Next release

---

## 3. Security Features

### Authentication & Authorization

**Default Security Posture:**
```toml
[orchestrator]
enabled = false              # Disabled by default
allow_public = false         # Loopback-only when enabled
token = ""                   # Must be explicitly set
require_client_cert = false  # Optional mTLS
```

**Bearer Token Authentication:**
- Minimum entropy: 256 bits (32 bytes)
- Generation: `openssl rand -base64 32`
- Storage: Environment variable or external secret manager
- Transmission: TLS 1.3 only

**TLS Configuration:**
```toml
[security]
enable_tls = true
tls_cert_path = "/etc/esnode/certs/server.crt"
tls_key_path = "/etc/esnode/certs/server.key"
tls_ca_path = "/etc/esnode/certs/ca.crt"
min_tls_version = "1.3"
cipher_suites = [
    "TLS_AES_256_GCM_SHA384",
    "TLS_CHACHA20_POLY1305_SHA256"
]
```

### Data Protection

**At Rest:**
- Configuration files: Filesystem permissions (600 for secrets)
- TSDB data: Optional AES-256 encryption
- Logs: Restricted to root/esnode user

**In Transit:**
- Metrics endpoint: TLS 1.3
- Control API: Mandatory TLS + mTLS option
- No plaintext credentials

**Data Minimization:**
- No PII collected
- No telemetry to vendor
- Metrics stay on-premise by default

### Audit Logging

All security-relevant events are logged:

```json
{
  "timestamp": "2026-02-09T00:36:20Z",
  "level": "AUDIT",
  "event_type": "AUTHENTICATION_FAILURE",
  "source_ip": "10.0.1.50",
  "user_agent": "curl/7.68.0",
  "endpoint": "/orchestrator/enforce",
  "reason": "invalid_token"
}
```

**Logged Events:**
- Authentication attempts (success/failure)
- Authorization failures
- Configuration changes
- Control plane actions
- Certificate changes
- Service start/stop

**Log Protection:**
- Append-only mode for audit logs
- Immutable after write
- Retention: 365 days minimum
- SIEM integration via syslog/JSON

---

## 4. Security Best Practices

### Deployment Hardening

**System User:**
```bash
# Create dedicated non-root user
useradd --system --no-create-home --shell /usr/sbin/nologin esnode

# Set file permissions
chown -R esnode:esnode /var/lib/esnode
chmod 700 /var/lib/esnode
chmod 600 /etc/esnode/esnode.toml
```

**Systemd Hardening:**
```ini
[Service]
User=esnode
Group=esnode
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ProtectKernelTunables=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictNamespaces=true
RestrictSUIDSGID=true
LockPersonality=true
```

**Network Isolation:**
```bash
# Firewall rules (example with ufw)
ufw allow from 10.0.0.0/8 to any port 9100 proto tcp
ufw deny 9100/tcp
```

**SELinux Policy:**
```bash
# Create custom SELinux policy module
# (Enterprise customers: contact support for production policy)
semanage fcontext -a -t esnode_exec_t '/usr/local/bin/esnode-core'
restorecon -v /usr/local/bin/esnode-core
```

### Secret Management

**DO NOT:**
- ❌ Commit secrets to Git
- ❌ Store in plaintext configuration
- ❌ Pass via command-line arguments
- ❌ Log secrets

**DO:**
- ✅ Use environment variables
- ✅ Use external secret managers (Vault, AWS Secrets Manager)
- ✅ Rotate credentials regularly
- ✅ Restrict file permissions

**Example: HashiCorp Vault Integration**
```bash
# Fetch token from Vault
export ESNODE_ORCHESTRATOR_TOKEN=$(vault kv get -field=token secret/esnode/orchestrator)

# Start agent
esnode-core
```

### Certificate Management

**Generation:**
```bash
# Generate private key (4096-bit RSA or Ed25519)
openssl genpkey -algorithm Ed25519 -out server.key

# Create CSR
openssl req -new -key server.key -out server.csr \
  -subj "/C=US/ST=CA/L=SF/O=YourCorp/CN=esnode.prod"

# Get certificate signed by corporate CA
# (Submit server.csr to your PKI team)
```

**Rotation:**
- Frequency: 90 days recommended
- Automation: Use cert-manager (Kubernetes) or ACME
- Zero-downtime: Graceful reload supported

---

## 5. Dependency Security

### Supply Chain Security

**Dependency Auditing:**
```bash
# Install cargo-audit
cargo install cargo-audit

# Run security audit
cargo audit
```

**Dependency Management:**
- Lock file (`Cargo.lock`) committed to Git
- Dependencies reviewed before updates
- Critical CVEs patched within 48 hours
- Monthly dependency refresh cycle

**Known Vulnerable Dependencies:**
None as of 2026-02-09.

### Build Provenance

**Reproducible Builds:**
- Deterministic compilation flags
- Pinned Rust toolchain version
- Build manifest with checksums

**Code Signing:**
```bash
# Sign binary (example for macOS)
codesign --sign "Developer ID Application" \
  --timestamp \
  --options runtime \
  target/release/esnode-core

# Verify signature
codesign --verify --deep --strict --verbose=2 \
  target/release/esnode-core
```

---

## 6. Security Testing

### Automated Security Checks

**CI/CD Pipeline:**
```yaml
# .github/workflows/security.yml
name: Security Audit
on: [push, pull_request]
jobs:
  security-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

**Static Analysis:**
- `cargo clippy` with security lints
- `cargo-geiger` for unsafe code detection
- `cargo-deny` for license compliance

### Manual Security Testing

**Recommended Tools:**
- **Fuzzing:** cargo-fuzz, AFL
- **SAST:** Semgrep, CodeQL
- **DAST:** OWASP ZAP, Burp Suite
- **Secrets Scanning:** TruffleHog, git-secrets

**Penetration Testing:**
- Annual third-party pentest recommended
- Scope: API endpoints, TLS configuration, auth bypass
- Report deadline: 90 days to remediate

---

## 7. Incident Response

### Security Incident Handling

**Phase 1: Detection**
- Monitoring: Failed auth attempts, unusual traffic
- Alerting: PagerDuty/Slack for critical events

**Phase 2: Containment**
```bash
# Immediately disable control API if compromised
sudo systemctl stop esnode-core

# Rotate compromised credentials
vault kv put secret/esnode/orchestrator token="$(openssl rand -base64 32)"
```

**Phase 3: Eradication**
- Patch vulnerable component
- Update to latest version
- Review audit logs

**Phase 4: Recovery**
- Restore from known-good backup
- Re-enable service with new credentials
- Monitor for 72 hours

**Phase 5: Post-Mortem**
- Root cause analysis
- Update runbooks
- Notify affected customers (if applicable)

---

## 8. Compliance & Certifications

### Standards Compliance

| Standard | Status | Notes |
|----------|--------|-------|
| **CIS Benchmarks** | Aligned | Hardening guide available |
| **NIST SP 800-53** | Compatible | Rev 5 controls |
| **ISO 27001** | Compatible | ISMS integration ready |
| **SOC 2 Type II** | Ready | Annual audit support |

### Data Residency

- **Default:** All data stays on local nodes
- **Export:** Customer-controlled (S3, GCS, on-prem)
- **Vendor Access:** Never (unless explicitly authorized for support)

### GDPR Compliance

- No personal data collected by default
- Data processing agreement available
- DPA template: https://esnode.io/dpa

---

## 9. Security Contacts

**Security Team:** security@esnode.co  
**PGP Fingerprint:** `ABCD 1234 EFGH 5678 IJKL 9012 MNOP 3456 QRST 7890`  
**Bug Bounty:** https://bugcrowd.com/esnode (Coming soon)  
**CVE Coordination:** security@esnode.co  

---

## 10. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-02-09 | Initial enterprise security policy |

---

**Maintained by:** ESNODE Security Team  
**Next Review:** 2026-08-09 (6 months)  
**Classification:** Public
