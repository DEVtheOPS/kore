# Security Policy

## Reporting a Vulnerability

We take the security of Kore seriously. If you discover a security vulnerability, please help us protect our users by reporting it responsibly.

### 🔒 For Critical Vulnerabilities

**DO NOT** create a public GitHub issue for critical security vulnerabilities.

Instead, please report them privately using one of these methods:

1. **GitHub Security Advisories (Preferred)**
   - Go to https://github.com/DEVtheOPS/kore/security/advisories/new
   - Provide detailed information about the vulnerability
   - We'll work with you on a fix and coordinate disclosure

2. **Email** (if GitHub Security Advisories is not available)
   - Send an email with details to the maintainers
   - Include "SECURITY" in the subject line
   - Encrypt sensitive details if possible

### ℹ️ For Non-Critical Security Issues

For security improvements or non-critical concerns, you can:
- Create a public issue using the "Security Vulnerability Report" template
- Start a discussion in GitHub Discussions

## What to Include

When reporting a vulnerability, please include:

- **Description** - Clear description of the vulnerability
- **Impact** - What could an attacker do with this vulnerability?
- **Steps to Reproduce** - How to reproduce the issue
- **Affected Versions** - Which versions are vulnerable
- **Suggested Fix** - If you have ideas on how to fix it
- **Your Environment** - OS, Kore version, Kubernetes version

## Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity
  - Critical: 7-14 days
  - High: 14-30 days
  - Medium/Low: 30-90 days

## Security Update Process

1. We'll acknowledge receipt of your report
2. We'll investigate and validate the vulnerability
3. We'll work on a fix and coordinate with you
4. We'll release a security update
5. We'll publicly disclose the vulnerability (coordinated disclosure)

## Supported Versions

We provide security updates for:

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | ✅ Yes             |
| 0.1.x   | ⚠️ Critical only   |
| < 0.1.0 | ❌ No              |

## Security Best Practices

When using Kore:

- **Keep Updated** - Always use the latest version
- **Secure Your Kubeconfig** - Kore stores kubeconfigs securely, but ensure they're encrypted at rest
- **Review Permissions** - Ensure RBAC permissions are appropriate for your use case
- **Network Security** - Use secure connections to your Kubernetes clusters
- **Audit Access** - Regularly review who has access to Kore and your clusters

## Known Security Advisories

We publish security advisories at:
- GitHub Security Advisories: https://github.com/DEVtheOPS/kore/security/advisories

## Acknowledgments

We appreciate security researchers who help make Kore safer. With your permission, we'll acknowledge your contribution in:
- The security advisory
- Release notes
- Our acknowledgments page

Thank you for helping keep Kore and our community safe! 🙏
