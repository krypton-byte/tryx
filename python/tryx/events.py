"""Event classes and payload types emitted by the Tryx runtime.

This module defines all event types that can be received during a WhatsApp
session lifecycle. Use the :meth:`Tryx.on` decorator to register handlers
for specific event classes.

Event categories:

- **Lifecycle**: ``EvConnected``, ``EvDisconnected``, ``EvLoggedOut``
- **Pairing**: ``EvPairingQrCode``, ``EvPairingCode``, ``EvPairSuccess``
- **Messaging**: ``EvMessage``, ``EvReceipt``, ``EvNotification``
- **Sync Actions**: ``EvPinUpdate``, ``EvMuteUpdate``, ``EvArchiveUpdate``
- **Contact/Profile**: ``EvPushNameUpdate``, ``EvPictureUpdate``, ``EvPresence``

Example::

    from tryx.client import Tryx
    from tryx.events import EvMessage

    app = Tryx(backend)

    @app.on(EvMessage)
    async def on_message(client, event):
        text = event.data.get_text()
        print(f"Received: {text}")
"""

from ._tryx import events  # type: ignore

for name in dir(events):  # type: ignore
    obj = getattr(events, name)  # type: ignore
    if isinstance(obj, type):
        globals()[name] = obj

__all__ = sorted(name for name, obj in globals().items() if isinstance(obj, type))
