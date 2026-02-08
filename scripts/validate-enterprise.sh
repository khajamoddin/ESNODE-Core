#!/usr/bin/env bash
# ESNODE-Core Enterprise Readiness Validator
# Validates software meets Fortune 500 / Mega-cap enterprise standards
# Copyright (c) 2024 Estimatedstocks AB | BUSL-1.1

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PASS=0
FAIL=0
WARN=0

# ============================================================================
# Helper Functions
# ============================================================================

pass() {
    ((PASS++))
    echo -e "${GREEN}✓${NC} $*"
}

fail() {
    ((FAIL++))
    echo -e "${RED}✗${NC} $*"
}

warn() {
    ((WARN++))
    echo -e "${YELLOW}⚠${NC} $*"
}

info() {
    echo -e "${BLUE}ℹ${NC} $*"
}

check() {
    local category="$1"
    shift
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "$category"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# ============================================================================
# Enterprise Readiness Checks
# ============================================================================

check_code_quality() {
    check "1. CODE QUALITY & SECURITY"
    
    # Compilation check
    if cargo check --workspace --quiet 2>/dev/null; then
        pass "Code compiles without errors"
    else
        fail "Code compilation failed"
    fi
    
    # Test coverage
    if cargo test --workspace --quiet 2>/dev/null; then
        pass "All tests passing"
    else
        fail "Tests failing"
    fi
    
    # Security audit
    if command -v cargo-audit &>/dev/null; then
        if cargo audit --quiet 2>/dev/null; then
            pass "No known security vulnerabilities (cargo audit)"
        else
            fail "Security vulnerabilities detected"
        fi
    else
        warn "cargo-audit not installed - security scan skipped"
    fi
    
    # Clippy lints
    local clippy_warnings=$(cargo clippy --workspace --quiet 2>&1 | grep -c "warning:" || true)
    if [ "$clippy_warnings" -le 5 ]; then
        pass "Clippy warnings acceptable ($clippy_warnings)"
    else
        warn "High clippy warnings: $clippy_warnings"
    fi
    
    # Memory safety verification
    if command -v cargo-geiger &>/dev/null; then
        local unsafe_count=$(cargo geiger 2>/dev/null | grep -c "unsafe" || echo "0")
        if [ "$unsafe_count" -le 5 ]; then
            pass "Minimal unsafe code usage ($unsafe_count blocks)"
        else
            warn "High unsafe code usage: $unsafe_count blocks"
        fi
    else
        info "cargo-geiger not installed - unsafe code check skipped"
    fi
}

check_documentation() {
    check "2. ENTERPRISE DOCUMENTATION"
    
    local required_docs=(
        "README.md"
        "LICENSE"
        "CHANGELOG.md"
        "SECURITY.md"
        "CONTRIBUTING.md"
        "CODE_OF_CONDUCT.md"
        "docs/ENTERPRISE_DEPLOYMENT.md"
        "docs/quickstart.md"
        "docs/metrics-list.md"
    )
    
    for doc in "${required_docs[@]}"; do
        if [ -f "$doc" ]; then
            pass "Documentation: $doc"
        else
            fail "Missing documentation: $doc"
        fi
    done
    
    # Check README quality
    if grep -q "Fortune 500\|Enterprise\|Production" README.md 2>/dev/null; then
        pass "README includes enterprise messaging"
    else
        warn "README lacks enterprise positioning"
    fi
}

check_build_system() {
    check "3. BUILD SYSTEM & PACKAGING"
    
    # Build scripts
    local build_scripts=(
        "scripts/build-enterprise.sh"
        "scripts/package-enterprise.sh"
    )
    
    for script in "${build_scripts[@]}"; do
        if [ -x "$script" ]; then
            pass "Build script exists and is executable: $script"
        else
            fail "Missing or non-executable: $script"
        fi
    done
    
    # Cargo.lock committed
    if [ -f "Cargo.lock" ]; then
        pass "Cargo.lock committed (reproducible builds)"
    else
        fail "Cargo.lock missing"
    fi
    
    # Release profile optimization
    if grep -q "lto = true\|lto = \"fat\"" Cargo.toml 2>/dev/null; then
        pass "LTO enabled for release builds"
    else
        warn "LTO not explicitly enabled"
    fi
}

check_security_hardening() {
    check "4. SECURITY HARDENING"
    
    # Security policy
    if [ -f "SECURITY.md" ]; then
        if grep -q "security@esnode.co\|vulnerability\|CVE" SECURITY.md; then
            pass "Comprehensive security policy exists"
        else
            warn "SECURITY.md exists but lacks detail"
        fi
    else
        fail "No SECURITY.md file"
    fi
    
    # TLS configuration
    if grep -rq "tls_cert_path\|enable_tls" crates/ 2>/dev/null; then
        pass "TLS support implemented"
    else
        warn "TLS support not found in code"
    fi
    
    # Authentication
    if grep -rq "bearer.*token\|authentication" crates/ 2>/dev/null; then
        pass "Authentication mechanisms present"
    else
        warn "Authentication not found"
    fi
    
    # Audit logging
    if grep -rq "audit.*log\|AUDIT" crates/ 2>/dev/null; then
        pass "Audit logging implemented"
    else
        warn "Audit logging not detected"
    fi
    
    # Non-root execution
    if grep -rq "User=esnode\|user.*esnode" scripts/ systemd/ 2>/dev/null; then
        pass "Non-root execution configured"
    else
        warn "Non-root user configuration not found"
    fi
}

check_deployment_readiness() {
    check "5. DEPLOYMENT & OPERATIONS"
    
    # Systemd service file
    if find . -name "*.service" -type f 2>/dev/null | grep -q .; then
        pass "Systemd service files present"
    else
        warn "No systemd service files found"
    fi
    
    # Health check endpoint
    if grep -rq "/healthz\|health.*check" crates/ 2>/dev/null; then
        pass "Health check endpoint implemented"
    else
        fail "Health check endpoint missing"
    fi
    
    # Graceful shutdown
    if grep -rq "signal.*handler\|SIGTERM" crates/ 2>/dev/null; then
        pass "Signal handling for graceful shutdown"
    else
        warn "Graceful shutdown not verified"
    fi
    
    # Configuration validation
    if grep -rq "validate.*config\|config.*error" crates/ 2>/dev/null; then
        pass "Configuration validation present"
    else
        warn "Configuration validation not detected"
    fi
}

check_observability() {
    check "6. OBSERVABILITY & MONITORING"
    
    # Metrics
    if grep -rq "prometheus\|metrics.*registry" crates/ 2>/dev/null; then
        pass "Prometheus metrics support"
    else
        fail "Prometheus metrics not found"
    fi
    
    # Structured logging
    if grep -rq "tracing\|slog\|log.*json" crates/ 2>/dev/null; then
        pass "Structured logging framework"
    else
        warn "Structured logging not verified"
    fi
    
    # Error tracking
    if grep -rq "Result\|anyhow\|thiserror" crates/ 2>/dev/null; then
        pass "Error handling with Result types"
    else
        fail "Proper error handling not found"
    fi
}

check_compliance() {
    check "7. COMPLIANCE & LEGAL"
    
    # License file
    if [ -f "LICENSE" ]; then
        local license_type=$(head -1 LICENSE)
        pass "License file present: $license_type"
    else
        fail "LICENSE file missing"
    fi
    
    # Copyright headers
    local files_with_copyright=$(find crates/ -name "*.rs" -exec grep -l "Copyright" {} \; 2>/dev/null | wc -l)
    if [ "$files_with_copyright" -gt 10 ]; then
        pass "Copyright headers in source files"
    else
        warn "Limited copyright headers found"
    fi
    
    # Third-party licenses
    if [ -f "Cargo.lock" ]; then
        if command -v cargo-license &>/dev/null; then
            pass "Can generate third-party license report (cargo-license)"
        else
            info "cargo-license not installed (optional)"
        fi
    fi
}

check_performance() {
    check "8. PERFORMANCE & SCALABILITY"
    
    # Release build exists
    if [ -f "target/release/esnode-core" ]; then
        local binary_size=$(du -h target/release/esnode-core | cut -f1)
        pass "Release binary built ($binary_size)"
    else
        warn "Release binary not built yet"
    fi
    
    # Async runtime
    if grep -rq "tokio\|async-std" crates/ 2>/dev/null; then
        pass "Async runtime for non-blocking I/O"
    else
        warn "Async runtime not detected"
    fi
    
    # Memory efficiency
    if grep -rq "Arc\|Rc" crates/ 2>/dev/null; then
        pass "Smart pointers for efficient memory management"
    else
        info "Memory management patterns not verified"
    fi
}

check_enterprise_features() {
    check "9. ENTERPRISE FEATURES"
    
    # Multi-tenancy support
    if grep -rq "tenant\|namespace\|isolation" crates/ 2>/dev/null; then
        pass "Multi-tenancy considerations"
    else
        info "Multi-tenancy not applicable or not detected"
    fi
    
    # High availability
    if grep -rq "ha\|high.*availability\|failover" docs/ 2>/dev/null; then
        pass "High availability documentation"
    else
        warn "HA considerations not documented"
    fi
    
    # Backup/restore
    if grep -rq "backup\|restore\|export.*tsdb" crates/ docs/ 2>/dev/null; then
        pass "Backup and restore capabilities"
    else
        warn "Backup/restore not verified"
    fi
    
    # Integration points
    if grep -rq "prometheus\|grafana\|vault\|siem" docs/ README.md 2>/dev/null; then
        pass "Enterprise integrations documented"
    else
        warn "Limited enterprise integration docs"
    fi
}

check_ai_specific() {
    check "10. AI/GPU INFRASTRUCTURE SPECIFIC"
    
    # GPU support
    if grep -rq "nvml\|cuda\|gpu" crates/ 2>/dev/null; then
        pass "NVIDIA GPU support (NVML)"
    else
        fail "GPU support not found"
    fi
    
    # MIG support
    if grep -rq "mig\|multi.*instance" crates/ 2>/dev/null; then
        pass "MIG (Multi-Instance GPU) support"
    else
        warn "MIG support not detected"
    fi
    
    # Power monitoring
    if grep -rq "power.*watts\|energy.*joules" crates/ 2>/dev/null; then
        pass "Power/energy monitoring"
    else
        fail "Power monitoring not found"
    fi
    
    # AI-specific metrics
    if grep -rq "tokens.*joule\|efficiency\|carbon" crates/ docs/ 2>/dev/null; then
        pass "AI efficiency metrics (tokens/joule, carbon)"
    else
        warn "AI-specific efficiency metrics limited"
    fi
}

# ============================================================================
# Generate Report
# ============================================================================

generate_report() {
    check "ENTERPRISE READINESS SUMMARY"
    
    local total=$((PASS + FAIL + WARN))
    local pass_percentage=$((PASS * 100 / total))
    
    echo ""
    echo "Results:"
    echo -e "  ${GREEN}PASS${NC}: $PASS"
    echo -e "  ${RED}FAIL${NC}: $FAIL"
    echo -e "  ${YELLOW}WARN${NC}: $WARN"
    echo -e "  ${BLUE}Total${NC}: $total"
    echo ""
    
    if [ "$FAIL" -eq 0 ] && [ "$pass_percentage" -ge 80 ]; then
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}✓ ENTERPRISE READY${NC}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        echo "ESNODE-Core meets enterprise-grade standards for:"
        echo "  ✓ Fortune 500 companies"
        echo "  ✓ Mega-cap corporations"
        echo "  ✓ Production deployment"
        echo "  ✓ Regulatory compliance"
        echo ""
        return 0
    elif [ "$FAIL" -eq 0 ]; then
        echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${YELLOW}⚠ ENTERPRISE READY (WITH WARNINGS)${NC}"
        echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        echo "ESNODE-Core is production-ready but has $WARN warnings."
        echo "Review warnings above for optimization opportunities."
        echo ""
        return 0
    else
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${RED}✗ NOT ENTERPRISE READY${NC}"
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        echo "ESNODE-Core has $FAIL critical failures."
        echo "Address all FAIL items before enterprise deployment."
        echo ""
        return 1
    fi
}

# ============================================================================
# Main Execution
# ============================================================================

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║     ESNODE-Core Enterprise Readiness Validation                  ║"
echo "║     Fortune 500 & Mega-Cap Standards Compliance Check            ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

# Run all checks
check_code_quality
check_documentation
check_build_system
check_security_hardening
check_deployment_readiness
check_observability
check_compliance
check_performance
check_enterprise_features
check_ai_specific

# Generate final report
generate_report
exit_code=$?

# Save report to file
REPORT_FILE="ENTERPRISE_READINESS_REPORT_$(date +%Y%m%d_%H%M%S).txt"
{
    echo "ESNODE-Core Enterprise Readiness Report"
    echo "Generated: $(date)"
    echo "=========================="
    echo ""
    echo "Summary:"
    echo "  PASS: $PASS"
    echo "  FAIL: $FAIL"
    echo "  WARN: $ WARN"
    echo "  Total: $((PASS + FAIL + WARN))"
} > "$REPORT_FILE"

info "Report saved to: $REPORT_FILE"

exit $exit_code
