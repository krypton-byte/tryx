from .types import MessageInfo
from .waproto.whatsapp_pb2 import Message as MessageProto

class Message:
    conversation: str | None
    caption: str | None
    message_info: MessageInfo

    def get_extended_text_message(self) -> str | None: ...
    def get_text(self) -> str | None: ...
    @property
    def raw_proto(self) -> MessageProto: ...

class PairingQrCode:
    code: str
    timeout: int
