import re

with open("src/backend/python_store.rs", "r") as f:
    content = f.read()

# Instead of regex, let's write a parser that finds the method signature and body,
# and generates the new positional version.
# Actually, the methods are:
methods = [
    # DeviceStore
    ("save", "device: &wacore::store::Device", "StoreResult<()>"),
    ("load", "", "StoreResult<Option<wacore::store::Device>>"),
    ("exists", "", "StoreResult<bool>"),
    ("create", "", "StoreResult<i32>"),
    # ProtocolStore
    ("get_sender_key_devices", "group_jid: &str", "StoreResult<Vec<(String, bool)>>"),
    ("set_sender_key_status", "group_jid: &str, entries: &[(&str, bool)]", "StoreResult<()>"),
    ("clear_sender_key_devices", "group_jid: &str", "StoreResult<()>"),
    ("delete_sender_key_device_rows", "device_jids: &[&str]", "StoreResult<()>"),
    ("clear_all_sender_key_devices", "", "StoreResult<()>"),
    ("get_lid_mapping", "lid: &str", "StoreResult<Option<LidPnMappingEntry>>"),
    ("get_pn_mapping", "phone: &str", "StoreResult<Option<LidPnMappingEntry>>"),
    ("put_lid_mapping", "entry: &LidPnMappingEntry", "StoreResult<()>"),
    ("get_all_lid_mappings", "", "StoreResult<Vec<LidPnMappingEntry>>"),
    ("save_base_key", "address: &str, message_id: &str, base_key: &[u8]", "StoreResult<()>"),
    ("has_same_base_key", "address: &str, message_id: &str, current_base_key: &[u8]", "StoreResult<bool>"),
    ("delete_base_key", "address: &str, message_id: &str", "StoreResult<()>"),
    ("update_device_list", "record: DeviceListRecord", "StoreResult<()>"),
    ("get_devices", "user: &str", "StoreResult<Option<DeviceListRecord>>"),
    ("delete_devices", "user: &str", "StoreResult<()>"),
    ("get_tc_token", "jid: &str", "StoreResult<Option<TcTokenEntry>>"),
    ("put_tc_token", "jid: &str, entry: &TcTokenEntry", "StoreResult<()>"),
    ("delete_tc_token", "jid: &str", "StoreResult<()>"),
    ("get_all_tc_token_jids", "", "StoreResult<Vec<String>>"),
    ("delete_expired_tc_tokens", "cutoff: i64", "StoreResult<u32>"),
    ("store_sent_message", "chat_jid: &str, message_id: &str, payload: &[u8]", "StoreResult<()>"),
    ("take_sent_message", "chat_jid: &str, message_id: &str", "StoreResult<Option<Vec<u8>>>"),
    ("delete_expired_sent_messages", "cutoff: i64", "StoreResult<u32>"),
    # AppSyncStore
    ("get_sync_key", "key_id: &[u8]", "StoreResult<Option<AppStateSyncKey>>"),
    ("set_sync_key", "key_id: &[u8], key: AppStateSyncKey", "StoreResult<()>"),
    ("get_version", "name: &str", "StoreResult<HashState>"),
    ("set_version", "name: &str, state: HashState", "StoreResult<()>"),
    ("put_mutation_macs", "name: &str, version: u64, mutations: &[AppStateMutationMAC]", "StoreResult<()>"),
    ("get_mutation_mac", "name: &str, index_mac: &[u8]", "StoreResult<Option<Vec<u8>>>"),
    ("delete_mutation_macs", "name: &str, index_macs: &[Vec<u8>]", "StoreResult<()>"),
    ("get_latest_sync_key_id", "", "StoreResult<Option<Vec<u8>>>"),
    # SignalStore
    ("put_identity", "address: &str, key: [u8; 32]", "StoreResult<()>"),
    ("load_identity", "address: &str", "StoreResult<Option<[u8; 32]>>"),
    ("delete_identity", "address: &str", "StoreResult<()>"),
    ("get_session", "address: &str", "StoreResult<Option<Bytes>>"),
    ("put_session", "address: &str, session: &[u8]", "StoreResult<()>"),
    ("delete_session", "address: &str", "StoreResult<()>"),
    ("store_prekey", "id: u32, record: &[u8], uploaded: bool", "StoreResult<()>"),
    ("load_prekey", "id: u32", "StoreResult<Option<Bytes>>"),
    ("remove_prekey", "id: u32", "StoreResult<()>"),
    ("get_max_prekey_id", "", "StoreResult<u32>"),
    ("store_signed_prekey", "id: u32, record: &[u8]", "StoreResult<()>"),
    ("load_signed_prekey", "id: u32", "StoreResult<Option<Vec<u8>>>"),
    ("load_all_signed_prekeys", "", "StoreResult<Vec<(u32, Vec<u8>)>>"),
    ("remove_signed_prekey", "id: u32", "StoreResult<()>"),
    ("put_sender_key", "address: &str, record: &[u8]", "StoreResult<()>"),
    ("get_sender_key", "address: &str", "StoreResult<Option<Vec<u8>>>"),
    ("delete_sender_key", "address: &str", "StoreResult<()>"),
    # MsgSecretStore
    ("put_msg_secrets", "entries: Vec<MsgSecretEntry>", "StoreResult<usize>"),
    ("get_msg_secret", "chat: &str, sender: &str, msg_id: &str", "StoreResult<Option<Vec<u8>>>"),
    ("delete_expired_msg_secrets", "cutoff: i64", "StoreResult<u32>"),
]
