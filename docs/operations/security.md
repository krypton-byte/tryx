# Security Practices

Secure your client around four risk zones: session state, operator controls, user input, and outbound behavior.

## Threat Model Snapshot

| Surface | Risk |
| --- | --- |
| Session backend | session hijack or forced re-pair |
| Command handlers | privilege escalation |
| Media pipelines | payload abuse / resource exhaustion |
| Outbound messaging | spam behavior and account penalties |

## Session Data Protection

- keep backend storage on protected, durable media
- restrict filesystem ACLs to runtime user only
- encrypt backups and protect backup key material

!!! danger "Do not expose backend files"
	Session artifacts are sensitive. Anyone with unrestricted access may impersonate the runtime.

## Credential and Token Hygiene

- never hardcode secrets in source
- load via environment or secret manager
- rotate integration credentials periodically

## Abuse and Ban Risk Reduction

- avoid repetitive high-frequency outbound sends
- enforce explicit user intent for auto-responses
- add anti-loop guardrails for client-to-client conversations

## Input Validation

- treat inbound message text and media metadata as untrusted
- validate command argument shape and length
- validate media type and size before decode/store

## Logging and Audit Safety

- redact sensitive message content by default
- log structured metadata instead of raw payload where possible
- split production audit logs from verbose debug traces

## Security Checklist Before Release

1. admin-only commands protected by allowlist
2. session backend path not world-readable
3. retry loops bounded and monitored
4. logs reviewed for accidental secrets

## Related Docs

- [Deployment Guide](deployment.md)
- [Reliability](reliability.md)
- [Privacy Namespace](../api/privacy.md)
