# Deployment Guide

Deploy Tryx bots safely in production with stable session storage and predictable restarts.

## Deployment Patterns

=== "Systemd Service"
    Best for single-host deployments.

    ```ini
    [Unit]
    Description=Tryx Bot
    After=network.target

    [Service]
    WorkingDirectory=/srv/tryx
    Environment=PYTHONUNBUFFERED=1
    ExecStart=/srv/tryx/.venv/bin/python bot.py
    Restart=always
    RestartSec=5
    User=tryx

    [Install]
    WantedBy=multi-user.target
    ```

=== "Container"
    Best for reproducible runtime images.

    ```dockerfile
    FROM python:3.12-slim

    WORKDIR /app
    COPY . .
    RUN pip install -U pip maturin && maturin develop

    CMD ["python", "bot.py"]
    ```

## Session Persistence Requirements

!!! warning "Critical"
    Keep backend/session data on durable storage. Stateless containers without mounted volumes will force re-pairing.

- Mount persistent volume for backend path.
- Use single active writer per backend path.
- Snapshot session data before upgrades.

??? info "Advanced: blue/green rollout note"
    During blue/green deploys, ensure only one color actively writes to the session backend.
    Dual-writer rollout against the same backend path can trigger stream replacement and churn.

## Release Checklist

1. Build in clean environment.
2. Run smoke command path in staging account.
3. Verify pairing/session reuse behavior.
4. Confirm logs/metrics are emitted.
5. Roll out with gradual traffic.

## Related Docs

- [Authentication Flow](../getting-started/authentication.md)
- [Reliability](reliability.md)
- [Troubleshooting](troubleshooting.md)
