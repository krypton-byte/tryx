# Tryx Documentation

<div class="tryx-hero tryx-fade-up">
  <h2>Build WhatsApp Automations With Rust Speed and Python Ergonomics</h2>
  <p>Tryx combines a Rust runtime core with a typed Python API so you can ship robust bots, integrations, and event-driven workflows without sacrificing performance.</p>
  <div class="tryx-pill-row">
    <span class="tryx-pill">Async-first</span>
    <span class="tryx-pill">Typed stubs (.pyi)</span>
    <span class="tryx-pill">PyO3 bindings</span>
    <span class="tryx-pill">Event-driven architecture</span>
  </div>
</div>

!!! tip "Documentation Mode"
    This site is designed as a full path: setup -> architecture -> API namespaces -> tutorials -> production operations.

## What You Can Do

<div class="tryx-grid">
  <div class="tryx-card">
    <h3>Messaging</h3>
    <p>Send text, photo, audio, document, video, GIF, and sticker content with a clean Python API.</p>
  </div>
  <div class="tryx-card">
    <h3>Realtime Events</h3>
    <p>Subscribe to rich event classes for messages, contact updates, sync actions, and lifecycle changes.</p>
  </div>
  <div class="tryx-card">
    <h3>Namespace Clients</h3>
    <p>Use dedicated namespaces for contacts, groups, newsletter, status, privacy, polls, presence, and more.</p>
  </div>
  <div class="tryx-card">
    <h3>Typed Development</h3>
    <p>Use complete Python stubs for editor intelligence, static checks, and better API discoverability.</p>
  </div>
</div>

## Choose Your Path

=== "I am New"
  1. [Installation](getting-started/installation.md)
  2. [Quick Start](getting-started/quickstart.md)
  3. [Authentication Flow](getting-started/authentication.md)
  4. [Client API Gateway](api/client.md)

=== "I am Building Features"
  1. [Client Namespaces](api/client.md)
  2. [Events API](api/events.md)
  3. [Types API](api/types.md)
  4. [Tutorials](tutorials/command-bot.md)

=== "I am Deploying"
  1. [Deployment Guide](operations/deployment.md)
  2. [Reliability](operations/reliability.md)
  3. [Troubleshooting](operations/troubleshooting.md)
  4. [Security](operations/security.md)

## Recommended Reading Path

1. Start with [Installation](getting-started/installation.md).
2. Follow [Quick Start](getting-started/quickstart.md) to build your first running client.
3. Understand pairing in [Authentication Flow](getting-started/authentication.md).
4. Learn internals in [Architecture](core-concepts/architecture.md) and [Event Model](core-concepts/event-model.md).
5. Jump into [Client API Gateway](api/client.md) and namespace deep dives.
6. Use [Tutorials](tutorials/command-bot.md) for implementation patterns.
7. Use [QnA](faq/qna.md) and [Troubleshooting](operations/troubleshooting.md) when debugging.
8. Finish with [Deployment](operations/deployment.md) and [Reliability](operations/reliability.md) before production.

## Project Scope

This documentation set focuses on the Python SDK experience first:

- Runtime setup and client lifecycle
- Event payload model and handler patterns
- API reference for client/events/types/helpers/wacore
- Performance, reliability, and security operations
- Practical tutorials and real-world QnA

Rust-internal details remain visible where they directly affect Python usage and behavior.

## Quick Technical Index

<div class="tryx-link-grid" markdown="1">
[Client Namespace Gateway](api/client.md)

All namespace clients mapped with deep-dive pages.

[Operational Readiness](operations/deployment.md)

Deployment, reliability, performance, security, and troubleshooting.

[Typed Contracts](api/types.md)

Shared value objects and type-first usage patterns.

[QnA](faq/qna.md)

Scenario-driven answers for practical problems.
</div>
