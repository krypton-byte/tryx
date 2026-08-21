"""Utility helpers for building payloads and handling common conversions.

This module provides static helper classes that simplify constructing
WhatsApp protocol objects. Each helper class focuses on a specific
domain (groups, newsletters, status, etc.).

Helper classes:

- :class:`NewsletterHelpers` — Newsletter message serialization and builders.
- :class:`GroupsHelpers` — Group invite and participant option construction.
- :class:`StatusHelpers` — Status privacy and send option builders.
- :class:`ChatstateHelpers` — Chat state enum value constructors.
- :class:`BlockingHelpers` — Blocklist identity matching utilities.
- :class:`PollsHelpers` — Poll creation and option builders.

Example::

    from tryx.helpers import GroupsHelpers
    from tryx.types import JID

    participant = GroupsHelpers.build_participant(
        jid=JID("1234567890", "s.whatsapp.net")
    )
"""

from ._tryx import helpers  # type: ignore

for name in dir(helpers):  # type: ignore
    obj = getattr(helpers, name)  # type: ignore
    if isinstance(obj, type):
        globals()[name] = obj

__all__ = [name for name in dir(helpers) if isinstance(getattr(helpers, name), type)]  # type: ignore
