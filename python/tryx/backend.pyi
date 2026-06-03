class StoreBase:
    """
    Base class for custom Python backends.
    All complex types are passed as JSON bytes.
    """
    async def put_identity(self, address: str, key: bytes) -> None:
        raise NotImplementedError()
    async def load_identity(self, address: str) -> bytes:
        raise NotImplementedError()
    async def delete_identity(self, address: str) -> None:
        raise NotImplementedError()
    async def get_session(self, address: str) -> bytes:
        raise NotImplementedError()
    async def put_session(self, address: str, session: bytes) -> None:
        raise NotImplementedError()
    async def delete_session(self, address: str) -> None:
        raise NotImplementedError()
    async def store_prekey(self, id: int, record: bytes, uploaded: bool) -> None:
        raise NotImplementedError()
    async def load_prekey(self, id: int) -> bytes:
        raise NotImplementedError()
    async def remove_prekey(self, id: int) -> None:
        raise NotImplementedError()
    async def get_max_prekey_id(self) -> int:
        raise NotImplementedError()
    async def store_signed_prekey(self, id: int, record: bytes) -> None:
        raise NotImplementedError()
    async def load_signed_prekey(self, id: int) -> bytes:
        raise NotImplementedError()
    async def load_all_signed_prekeys(self) -> bytes:
        raise NotImplementedError()
    async def remove_signed_prekey(self, id: int) -> None:
        raise NotImplementedError()
    async def put_sender_key(self, address: str, record: bytes) -> None:
        raise NotImplementedError()
    async def get_sender_key(self, address: str) -> bytes:
        raise NotImplementedError()
    async def delete_sender_key(self, address: str) -> None:
        raise NotImplementedError()
    async def get_sync_key(self, key_id: bytes) -> bytes:
        raise NotImplementedError()
    async def set_sync_key(self, key_id: bytes, key: bytes) -> None:
        raise NotImplementedError()
    async def get_version(self, name: str) -> bytes:
        raise NotImplementedError()
    async def set_version(self, name: str, state: bytes) -> None:
        raise NotImplementedError()
    async def put_mutation_macs(self, name: str, version: int, mutations: bytes) -> None:
        raise NotImplementedError()
    async def get_mutation_mac(self, name: str, index_mac: bytes) -> bytes:
        raise NotImplementedError()
    async def delete_mutation_macs(self, name: str, index_macs: bytes) -> None:
        raise NotImplementedError()
    async def get_latest_sync_key_id(self) -> bytes:
        raise NotImplementedError()
    async def save(self, device: bytes) -> None:
        raise NotImplementedError()
    async def load(self) -> bytes:
        raise NotImplementedError()
    async def exists(self) -> bool:
        raise NotImplementedError()
    async def create(self) -> bytes:
        raise NotImplementedError()
    async def get_sender_key_devices(self, group_jid: str) -> bytes:
        raise NotImplementedError()
    async def set_sender_key_status(self, group_jid: str, entries: bytes) -> None:
        raise NotImplementedError()
    async def clear_sender_key_devices(self, group_jid: str) -> None:
        raise NotImplementedError()
    async def delete_sender_key_device_rows(self, device_jids: bytes) -> None:
        raise NotImplementedError()
    async def clear_all_sender_key_devices(self) -> None:
        raise NotImplementedError()
    async def get_lid_mapping(self, lid: str) -> bytes:
        raise NotImplementedError()
    async def get_pn_mapping(self, phone: str) -> bytes:
        raise NotImplementedError()
    async def put_lid_mapping(self, entry: bytes) -> None:
        raise NotImplementedError()
    async def get_all_lid_mappings(self) -> bytes:
        raise NotImplementedError()
    async def save_base_key(self, address: str, message_id: str, base_key: bytes) -> None:
        raise NotImplementedError()
    async def has_same_base_key(self, address: str, message_id: str, current_base_key: bytes) -> bool:
        raise NotImplementedError()
    async def delete_base_key(self, address: str, message_id: str) -> None:
        raise NotImplementedError()
    async def update_device_list(self, record: bytes) -> None:
        raise NotImplementedError()
    async def get_devices(self, user: str) -> bytes:
        raise NotImplementedError()
    async def delete_devices(self, user: str) -> None:
        raise NotImplementedError()
    async def get_tc_token(self, jid: str) -> bytes:
        raise NotImplementedError()
    async def put_tc_token(self, jid: str, entry: bytes) -> None:
        raise NotImplementedError()
    async def delete_tc_token(self, jid: str) -> None:
        raise NotImplementedError()
    async def get_all_tc_token_jids(self) -> bytes:
        raise NotImplementedError()
    async def delete_expired_tc_tokens(self, cutoff: int) -> int:
        raise NotImplementedError()
    async def store_sent_message(self, chat_jid: str, message_id: str, payload: bytes) -> None:
        raise NotImplementedError()
    async def take_sent_message(self, chat_jid: str, message_id: str) -> bytes:
        raise NotImplementedError()
    async def delete_expired_sent_messages(self, cutoff: int) -> int:
        raise NotImplementedError()
    async def put_msg_secrets(self, entries: bytes) -> int:
        raise NotImplementedError()
    async def get_msg_secret(self, chat: str, sender: str, msg_id: str) -> bytes:
        raise NotImplementedError()
    async def delete_expired_msg_secrets(self, cutoff: int) -> int:
        raise NotImplementedError()
