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

    def test_single_arg_and_parse(self) -> None:
        jid = JID("628123456789@s.whatsapp.net")
        assert jid.user == "628123456789"
        assert jid.server == "s.whatsapp.net"
        assert str(jid) == "628123456789@s.whatsapp.net"
        assert jid.is_pn
        assert not jid.is_group

        group = JID.parse("120363001234567@g.us")
        assert group.is_group
        assert not group.is_pn

        plain = JID("628123456789")
        assert plain.server == "s.whatsapp.net"

    def test_equality_and_hashing(self) -> None:
        a = JID("1234567890", "s.whatsapp.net")
        b = JID("1234567890@s.whatsapp.net")
        c = JID("9999999999@s.whatsapp.net")
        assert a == b
        assert a != c
        assert hash(a) == hash(b)
        mapping = {a: "ok"}
        assert mapping[b] == "ok"
        assert a.is_same_chat_as("1234567890@s.whatsapp.net")
