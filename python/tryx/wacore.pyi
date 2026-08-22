"""Low-level protocol-facing data types exposed by Tryx."""

from datetime import datetime
from enum import Enum

from .types import JID

class MediaType(Enum):
    """Supported media categories for upload and download operations."""

    Image = 1
    Video = 2
    Audio = 3
    Document = 4
    History = 5
    AppState = 6
    Sticker = 7
    StickerPack = 8
    LinkThumbnail = 9

class NodeValue:
    """Represents a node attribute value.

    A node value can be a plain string or a JID object.
    """

    def __init__(self, value: str) -> None:
        """Create a NodeValue from a string.

        Args:
            value: Initial string value.

        Example::

            val = NodeValue('hello')
            print(val.value)  # 'hello'
        """
        ...

    @staticmethod
    def jid(value: JID) -> NodeValue:
        """Create a NodeValue from a JID.

        Args:
            value: JID to wrap.

        Returns:
            NodeValue wrapping the JID.

        Example::

            val = NodeValue.jid(JID('123', 's.whatsapp.net'))
            print(val.value)  # JID object
        """
        ...

    def set_string(self, value: str) -> None:
        """Replace the current value with a string.

        Args:
            value: New string value.

        Example::

            val = NodeValue('old')
            val.set_string('new')
        """
        ...

    def set_jid(self, value: JID) -> None:
        """Replace the current value with a JID.

        Args:
            value: New JID value.

        Example::

            val = NodeValue('default')
            val.set_jid(JID('123', 's.whatsapp.net'))
        """
        ...

    @property
    def value(self) -> str | JID:
        """Return the current value as either a string or JID.

        Returns:
            The attribute value (str or JID).

        Raises:
            TypeError: If the internal type is neither str nor JID.
        """
        ...

    @value.setter
    def value(self, value: str | JID) -> None:
        """Set the current value to a string or JID.

        Args:
            value: New value (string or JID).

        Raises:
            TypeError: If *value* is not a str or JID.
        """
        ...

class NodeContent:
    """Represents node payload content.

    Content can be raw bytes, a string, or a list of child nodes.
    """

    @property
    def value(self) -> bytes | str | list[Node]:
        """Return the current content value.

        Returns:
            Content as bytes, string, or list of child Nodes.
        """
        ...

    def is_bytes(self) -> bool:
        """Return True if content is bytes."""
        ...

    def is_string(self) -> bool:
        """Return True if content is a string."""
        ...

    def is_nodes(self) -> bool:
        """Return True if content is a list of nodes."""
        ...

class Attrs:
    """Single attribute entry used in a Node."""

    key: str
    value: NodeValue

    def __init__(self, key: str, value: NodeValue) -> None:
        """Create a node attribute pair.

        Args:
            key: Attribute name.
            value: Attribute value.

        Example::

            attr = Attrs('to', NodeValue.jid(JID('123', 's.whatsapp.net')))
        """
        ...

class Node:
    """Generic protocol node structure used by low-level event payloads."""

    tag: str
    attrs: list[Attrs]
    content: NodeContent | None

    def __init__(
        self,
        tag: str,
        attrs: list[Attrs],
        content: NodeContent | None,
    ) -> None:
        """Create a protocol node object.

        Args:
            tag: Node tag name.
            attrs: List of attribute entries.
            content: Optional payload content.

        Example::

            node = Node(
                tag='message',
                attrs=[Attrs('to', NodeValue.jid(JID('123', 's.whatsapp.net')))],
                content=None,
            )
        """
        ...

class KeyIndexInfo:
    """Metadata for device key index updates."""

    timestamp: int
    signed_bytes: bytes | None

class BusinessSubscription:
    """Business subscription status entry attached to sync events."""

    id: str
    status: str
    expiration_date: datetime | None
    creation_time: datetime | None
