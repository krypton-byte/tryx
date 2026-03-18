from .types import JID, MessageInfo
from .waproto.whatsapp_pb2 import Message as MessageProto


class EvConnected:
    pass

class EvDisconnected:
    pass

class EvLoggedOut:
    on_connect: bool
    reason: str

class EvPairSuccess:
    id: JID
    lid: JID
    business_name: str
    platform: str

class EvPairError:
    id: JID
    lid: JID
    business_name: str
    platform: str
    error: str

class EvPairingQrCode:
    code: str
    timeout: int

class EvMessage:
    conversation: str | None
    caption: str | None
    message_info: MessageInfo

    def get_extended_text_message(self) -> str | None: ...
    def get_text(self) -> str | None: ...
    @property
    def raw_proto(self) -> MessageProto: ...

class EvQrScannedWithoutMultidevice:
    pass