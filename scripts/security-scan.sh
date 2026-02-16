#!/usr/bin/env bash
# Run the same security checks that CI runs

set -e

echo "Running Rust Security Scan..."
echo "=============================="
echo ""

# Move to src-tauri directory
cd "$(dirname "$0")/../src-tauri" || exit 1

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track exit codes
audit_exit=0
deny_exit=0
clippy_exit=0

echo "1. Running cargo audit..."
echo "-------------------------"
if cargo audit; then
    echo -e "${GREEN}✓ cargo audit: PASS${NC}"
else
    audit_exit=$?
    echo -e "${RED}✗ cargo audit: FAIL (exit $audit_exit)${NC}"
fi
echo ""

echo "2. Running cargo deny check advisories..."
echo "-----------------------------------------"
if cargo deny check advisories; then
    echo -e "${GREEN}✓ cargo deny: PASS${NC}"
else
    deny_exit=$?
    echo -e "${RED}✗ cargo deny: FAIL (exit $deny_exit)${NC}"
fi
echo ""

echo "3. Running cargo clippy security lints..."
echo "-----------------------------------------"
if cargo clippy --all-targets --all-features -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic; then
    echo -e "${GREEN}✓ cargo clippy: PASS${NC}"
else
    clippy_exit=$?
    echo -e "${RED}✗ cargo clippy: FAIL (exit $clippy_exit)${NC}"
fi
echo ""

# Summary
echo "=============================="
echo "Summary:"
echo "=============================="
if [ $audit_exit -eq 0 ]; then
    echo -e "cargo audit:   ${GREEN}PASS${NC}"
else
    echo -e "cargo audit:   ${RED}FAIL (exit $audit_exit)${NC}"
fi

if [ $deny_exit -eq 0 ]; then
    echo -e "cargo deny:    ${GREEN}PASS${NC}"
else
    echo -e "cargo deny:    ${RED}FAIL (exit $deny_exit)${NC}"
fi

if [ $clippy_exit -eq 0 ]; then
    echo -e "cargo clippy:  ${GREEN}PASS${NC}"
else
    echo -e "cargo clippy:  ${RED}FAIL (exit $clippy_exit)${NC}"
fi
echo ""

# Exit with error if any check failed
if [ $audit_exit -ne 0 ] || [ $deny_exit -ne 0 ] || [ $clippy_exit -ne 0 ]; then
    echo -e "${RED}Security scan FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}All security checks PASSED!${NC}"
    exit 0
fi
