"""Client API gateway for WhatsApp automation.

This module provides the main :class:`Tryx` entry point that wires together
the Rust runtime, event dispatcher, and namespace clients. It is the
recommended way to start a WhatsApp automation project.

Typical usage::

    from tryx.backend import SqliteStore
    from tryx.client import Tryx

    app = Tryx(SqliteStore("session.db"))

    @app.on(EvMessage)
    async def on_message(client, event):
        ...

    import asyncio
    asyncio.run(app.run())
"""

from ._tryx import client  # type: ignore

for name in dir(client):  # type: ignore
    obj = getattr(client, name)  # type: ignore
    if isinstance(obj, type):
        globals()[name] = obj

__all__ = [name for name in dir(client) if isinstance(getattr(client, name), type)]  # type: ignore
