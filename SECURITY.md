# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Tryx, please report it
responsibly. **Do not open a public GitHub issue for security vulnerabilities.**

### How to Report

1. **Email**: Send a detailed report to
   [krypton-byte](https://github.com/krypton-byte) via GitHub's private
   vulnerability reporting feature, or open a
   [Security Advisory](https://github.com/krypton-byte/tryx/security/advisories/new).

2. **Include**:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact assessment
   - Any suggested fix (if available)

3. **Response time**: We aim to acknowledge reports within **72 hours**.

## Scope

The following are considered in-scope for security reports:

- **Protocol handling** — Rust core logic for Signal protocol, encryption,
  and key exchange
- **PyO3 bridge** — Type conversion, GIL management, and error marshaling
  between Rust and Python
- **Session storage** — SQLite, FFI, and Python store backends
- **Media processing** — Upload, download, and transcoding paths
- **Authentication** — Pairing, session persistence, and credential handling

## Out of Scope

- Issues in third-party dependencies (report upstream)
- Denial of service against WhatsApp servers
- Social engineering attacks
- Issues requiring physical access to the target device

## Disclosure Policy

- We follow a **coordinated disclosure** process.
- We will work with you to understand and address the issue before any
  public disclosure.
- Once a fix is released, we will publicly acknowledge the vulnerability
  and credit the reporter (unless anonymity is requested).

## Supported Versions

| Version | Supported |
| --- | --- |
| 1.x | ✅ Active |
| 0.x | ❌ End of life |

## Best Practices for Users

- Keep Tryx updated to the latest version.
- Use environment variables or a secrets manager for sensitive configuration.
- Restrict admin-level bot commands to authorized users.
- Monitor `EvTemporaryBan` events for rate-limit violations.
- Use a dedicated WhatsApp account for automation (not your personal account).
