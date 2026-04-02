"""Unit tests for Tryx type bindings."""

from __future__ import annotations

from tryx.types import JID


class TestJID:
    """Tests for the JID type."""

    def test_construction(self) -> None:
        jid = JID("1234567890", "s.whatsapp.net")
        assert jid.user == "1234567890"
        assert jid.server == "s.whatsapp.net"

    def test_repr(self) -> None:
        jid = JID("1234567890", "s.whatsapp.net")
        r = repr(jid)
        assert "1234567890" in r
        assert "s.whatsapp.net" in r

    def test_different_servers(self) -> None:
        jid_user = JID("123", "s.whatsapp.net")
        jid_group = JID("123456", "g.us")
        assert jid_user.server == "s.whatsapp.net"
        assert jid_group.server == "g.us"

    def test_empty_user(self) -> None:
        jid = JID("", "s.whatsapp.net")
        assert jid.user == ""
