from .waproto.whatsapp_pb2 import Message as MessageProto


class NewsletterHelpers:
    @staticmethod
    def parse_message(data: bytes) -> MessageProto: ...
    @staticmethod
    def serialize_message(message: MessageProto) -> bytes: ...
    @staticmethod
    def build_text_message(text: str) -> MessageProto: ...
