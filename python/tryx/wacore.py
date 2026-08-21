"""Low-level protocol-facing data types exposed by Tryx.

This module provides the core protocol types that underpin the WhatsApp
Web wire format. Most users will not need to interact with these types
directly — they are used internally by the event system and client APIs.

Key types:

- :class:`MediaType` — Supported media categories (Image, Video, Audio, etc.).
- :class:`Node` — Protocol node representation (XML-like tree structure).
- :class:`NodeValue` — Attribute value (string or JID).
- :class:`NodeContent` — Node payload (bytes, string, or child nodes).
- :class:`Attrs` — Single attribute entry used in a Node.
- :class:`KeyIndexInfo` — Encryption key index metadata.
- :class:`BusinessSubscription` — Business account subscription info.

Example::

    from tryx.wacore import Node, MediaType

    # Media type enum
    media = MediaType.Image

    # Protocol node (advanced usage)
    node = Node(tag="message", attrs=[], content=None)
"""

from ._tryx import wacore  # type: ignore

for name in dir(wacore):  # type: ignore
    obj = getattr(wacore, name)  # type: ignore
    if isinstance(obj, type):
        globals()[name] = obj

__all__ = [name for name in dir(wacore) if isinstance(getattr(wacore, name), type)]  # type: ignore
