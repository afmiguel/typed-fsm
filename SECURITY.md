# Security Policy

## Supported Versions

Currently supported versions of typed-fsm:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in typed-fsm, please report it responsibly.

### How to Report

**Please do NOT open a public issue for security vulnerabilities.**

Instead, please report security issues by:

1. **Email:** Send details to the maintainer via GitHub
2. **GitHub Security Advisory:** Use the "Security" tab on the repository to privately report vulnerabilities

### What to Include

When reporting a vulnerability, please include:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)
- Your contact information

### Response Timeline

- **Acknowledgment:** Within 48 hours
- **Initial Assessment:** Within 1 week
- **Fix Timeline:** Depends on severity
  - Critical: Within 7 days
  - High: Within 30 days
  - Medium: Within 90 days
  - Low: Best effort

### Disclosure Policy

- We will acknowledge your report within 48 hours
- We will provide an estimated timeline for a fix
- We will notify you when the vulnerability is fixed
- We will credit you in the security advisory (unless you prefer to remain anonymous)

## Security Considerations

### Safe by Design

typed-fsm is designed with security in mind:

- **No unsafe code:** The library does not use `unsafe` blocks
- **No dependencies:** Zero dependencies means minimal attack surface
- **Compile-time safety:** Type system prevents invalid states at compile time
- **no_std compatible:** Suitable for security-critical embedded systems

### Best Practices

When using typed-fsm:

- Validate all external inputs before processing as events
- Use appropriate access controls for state machine contexts
- Follow Rust security best practices
- Keep your Rust toolchain updated

## Known Issues

No known security vulnerabilities at this time.

Check [GitHub Security Advisories](https://github.com/afmiguel/typed-fsm/security/advisories) for updates.

## Questions?

For security-related questions that are not vulnerabilities, please open a regular issue on GitHub.
