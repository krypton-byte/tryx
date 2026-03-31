# Security Practices

## Session Data Protection

- store backend database on protected storage
- restrict filesystem permissions
- encrypt backups where possible

## Credential and Token Hygiene

- never hardcode secrets in handlers
- load sensitive values from environment variables
- rotate credentials for external integrations

## Abuse and Ban Risk Reduction

- avoid spammy repetitive sends
- respect rate limits and user consent
- implement anti-loop logic in auto-reply bots

## Input Validation

- treat all inbound message content as untrusted
- sanitize command input before shell/system usage
- validate file type and size before processing media

## Logging Safety

- avoid logging full private content by default
- redact user identifiers where required
- separate debug logs from production audit logs
