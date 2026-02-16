# Security Scan Script

## Overview

This script runs the same security checks that the CI pipeline runs, allowing you to test locally before pushing.

## Prerequisites

1. **cargo-audit** - Install with:
   ```bash
   cargo install cargo-audit
   ```

2. **cargo-deny** - Install with:
   ```bash
   cargo install cargo-deny
   ```

## Usage

From the project root:

```bash
./scripts/security-scan.sh
```

Or from anywhere:

```bash
/path/to/kore/scripts/security-scan.sh
```

## What it checks

The script runs three security scans:

1. **cargo audit** - Checks for known security vulnerabilities in dependencies
2. **cargo deny** - Checks security advisories with custom ignore rules
3. **cargo clippy** - Runs strict clippy lints to prevent unsafe code patterns:
   - `-W clippy::unwrap_used` - No `.unwrap()` in production code
   - `-W clippy::expect_used` - No `.expect()` in production code
   - `-W clippy::panic` - No `panic!()` in production code

## Exit codes

- **0** - All checks passed ✅
- **1** - One or more checks failed ❌

## Individual commands

You can also run each check individually from `src-tauri/`:

```bash
# Audit dependencies
cargo audit

# Check advisories with deny
cargo deny check advisories

# Run security lints
cargo clippy --all-targets --all-features -- \
  -D warnings \
  -W clippy::unwrap_used \
  -W clippy::expect_used \
  -W clippy::panic
```

## Ignored Advisories

See `src-tauri/deny.toml` and `src-tauri/.cargo/audit.toml` for the list of ignored security advisories. These are primarily unmaintained dependencies from the Tauri framework that will be resolved when Tauri updates.
