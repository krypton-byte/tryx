use std::ffi::{c_char, c_void, CString};
use std::ptr;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use libloading::{Library, Symbol};

use wacore::appstate::hash::HashState;
use wacore::store::traits::*;
use wacore::store::error::{Result as StoreResult, StoreError};
use wacore_appstate::processor::AppStateMutationMAC;

#[repr(C)]
pub struct TryxBuffer {
    pub data: *mut u8,
    pub len: usize,
}

impl TryxBuffer {
    fn into_vec(self) -> Vec<u8> {
        if self.data.is_null() || self.len == 0 {
            return Vec::new();
        }
        unsafe { Vec::from_raw_parts(self.data, self.len, self.len) }
    }
}

type ConnectFn = unsafe extern "C" fn(*const c_char, *mut *mut c_void) -> i32;
type DestroyFn = unsafe extern "C" fn(*mut c_void);
type FreeBufferFn = unsafe extern "C" fn(TryxBuffer);

type PutIdentityFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const u8, usize) -> i32;
type LoadIdentityFn = unsafe extern "C" fn(*mut c_void, *const c_char, *mut TryxBuffer) -> i32;
type DeleteIdentityFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> i32;

type GetSessionFn = unsafe extern "C" fn(*mut c_void, *const c_char, *mut TryxBuffer) -> i32;
type PutSessionFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const u8, usize) -> i32;
type DeleteSessionFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> i32;

type StorePrekeyFn = unsafe extern "C" fn(*mut c_void, u32, *const u8, usize, i32) -> i32;
type LoadPrekeyFn = unsafe extern "C" fn(*mut c_void, u32, *mut TryxBuffer) -> i32;
type RemovePrekeyFn = unsafe extern "C" fn(*mut c_void, u32) -> i32;
type GetMaxPrekeyIdFn = unsafe extern "C" fn(*mut c_void, *mut u32) -> i32;

type StoreSignedPrekeyFn = unsafe extern "C" fn(*mut c_void, u32, *const u8, usize) -> i32;
type LoadSignedPrekeyFn = unsafe extern "C" fn(*mut c_void, u32, *mut TryxBuffer) -> i32;
type RemoveSignedPrekeyFn = unsafe extern "C" fn(*mut c_void, u32) -> i32;

type PutSenderKeyFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const u8, usize) -> i32;
type GetSenderKeyFn = unsafe extern "C" fn(*mut c_void, *const c_char, *mut TryxBuffer) -> i32;
type DeleteSenderKeyFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> i32;

type GetSyncKeyFn = unsafe extern "C" fn(*mut c_void, *const u8, usize, *mut TryxBuffer) -> i32;
type SetSyncKeyFn = unsafe extern "C" fn(*mut c_void, *const u8, usize, *const u8, usize) -> i32;
type GetVersionFn = unsafe extern "C" fn(*mut c_void, *const c_char, *mut TryxBuffer) -> i32;
type SetVersionFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const u8, usize) -> i32;
type GetLatestSyncKeyIdFn = unsafe extern "C" fn(*mut c_void, *mut TryxBuffer) -> i32;

type SaveDeviceFn = unsafe extern "C" fn(*mut c_void, *const u8, usize) -> i32;
type LoadDeviceFn = unsafe extern "C" fn(*mut c_void, *mut TryxBuffer) -> i32;
type DeviceExistsFn = unsafe extern "C" fn(*mut c_void, *mut i32) -> i32;
type CreateDeviceFn = unsafe extern "C" fn(*mut c_void, *mut i32) -> i32;

type CallFn = unsafe extern "C" fn(*mut c_void, u32, *const u8, usize, *mut TryxBuffer) -> i32;

struct FfiLib {
    _lib: Library,
    handle: *mut c_void,
    destroy: DestroyFn,
    free_buffer: FreeBufferFn,

    put_identity: PutIdentityFn,
    load_identity: LoadIdentityFn,
    delete_identity: DeleteIdentityFn,

    get_session: GetSessionFn,
    put_session: PutSessionFn,
    delete_session: DeleteSessionFn,

    store_prekey: StorePrekeyFn,
    load_prekey: LoadPrekeyFn,
    remove_prekey: RemovePrekeyFn,
    get_max_prekey_id: GetMaxPrekeyIdFn,

    store_signed_prekey: StoreSignedPrekeyFn,
    load_signed_prekey: LoadSignedPrekeyFn,
    remove_signed_prekey: RemoveSignedPrekeyFn,

    put_sender_key: PutSenderKeyFn,
    get_sender_key: GetSenderKeyFn,
    delete_sender_key: DeleteSenderKeyFn,

    get_sync_key: GetSyncKeyFn,
    set_sync_key: SetSyncKeyFn,
    get_version: GetVersionFn,
    set_version: SetVersionFn,
    get_latest_sync_key_id: GetLatestSyncKeyIdFn,

    save_device: SaveDeviceFn,
    load_device: LoadDeviceFn,
    device_exists: DeviceExistsFn,
    create_device: CreateDeviceFn,

    call: CallFn,
}

unsafe impl Send for FfiLib {}
unsafe impl Sync for FfiLib {}

impl Drop for FfiLib {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { (self.destroy)(self.handle) };
        }
    }
}

fn make_err(msg: impl Into<String>) -> StoreError {
    StoreError::Database(msg.into().into())
}

#[derive(Clone)]
pub struct FfiBridgeStore {
    ffi: Arc<FfiLib>,
}

impl FfiBridgeStore {
    pub async fn connect(lib_path: &str, config_json: &str) -> StoreResult<Self> {
        let lib = unsafe { Library::new(lib_path) }.map_err(|e| make_err(e.to_string()))?;
        
        let config_str = CString::new(config_json).unwrap();

        let handle = unsafe {
            let connect: Symbol<ConnectFn> = lib.get(b"tryx_store_connect\0").map_err(|e| make_err(e.to_string()))?;
            let mut handle: *mut c_void = ptr::null_mut();
            if connect(config_str.as_ptr(), &mut handle) != 0 {
                return Err(make_err("Failed to connect to FFI store"));
            }
            handle
        };

        let ffi = unsafe {
            FfiLib {
                destroy: *lib.get(b"tryx_store_destroy\0").unwrap(),
                free_buffer: *lib.get(b"tryx_store_free_buffer\0").unwrap(),
                put_identity: *lib.get(b"tryx_store_put_identity\0").unwrap(),
                load_identity: *lib.get(b"tryx_store_load_identity\0").unwrap(),
                delete_identity: *lib.get(b"tryx_store_delete_identity\0").unwrap(),
                get_session: *lib.get(b"tryx_store_get_session\0").unwrap(),
                put_session: *lib.get(b"tryx_store_put_session\0").unwrap(),
                delete_session: *lib.get(b"tryx_store_delete_session\0").unwrap(),
                store_prekey: *lib.get(b"tryx_store_store_prekey\0").unwrap(),
                load_prekey: *lib.get(b"tryx_store_load_prekey\0").unwrap(),
                remove_prekey: *lib.get(b"tryx_store_remove_prekey\0").unwrap(),
                get_max_prekey_id: *lib.get(b"tryx_store_get_max_prekey_id\0").unwrap(),
                store_signed_prekey: *lib.get(b"tryx_store_store_signed_prekey\0").unwrap(),
                load_signed_prekey: *lib.get(b"tryx_store_load_signed_prekey\0").unwrap(),
                remove_signed_prekey: *lib.get(b"tryx_store_remove_signed_prekey\0").unwrap(),
                put_sender_key: *lib.get(b"tryx_store_put_sender_key\0").unwrap(),
                get_sender_key: *lib.get(b"tryx_store_get_sender_key\0").unwrap(),
                delete_sender_key: *lib.get(b"tryx_store_delete_sender_key\0").unwrap(),
                get_sync_key: *lib.get(b"tryx_store_get_sync_key\0").unwrap(),
                set_sync_key: *lib.get(b"tryx_store_set_sync_key\0").unwrap(),
                get_version: *lib.get(b"tryx_store_get_version\0").unwrap(),
                set_version: *lib.get(b"tryx_store_set_version\0").unwrap(),
                get_latest_sync_key_id: *lib.get(b"tryx_store_get_latest_sync_key_id\0").unwrap(),
                save_device: *lib.get(b"tryx_store_save_device\0").unwrap(),
                load_device: *lib.get(b"tryx_store_load_device\0").unwrap(),
                device_exists: *lib.get(b"tryx_store_device_exists\0").unwrap(),
                create_device: *lib.get(b"tryx_store_create_device\0").unwrap(),
                call: *lib.get(b"tryx_store_call\0").unwrap(),
                _lib: lib,
                handle,
            }
        };

        Ok(Self { ffi: Arc::new(ffi) })
    }

    fn call(&self, op: u32, args: serde_json::Value) -> StoreResult<Option<Vec<u8>>> {
        let input = serde_json::to_vec(&args).unwrap();
        let mut out = TryxBuffer { data: ptr::null_mut(), len: 0 };
        let res = unsafe { (self.ffi.call)(self.ffi.handle, op, input.as_ptr(), input.len(), &mut out) };
        if res == 0 {
            let vec = out.into_vec();
            if vec.is_empty() { Ok(None) } else { Ok(Some(vec)) }
        } else {
            Err(make_err("FFI call failed"))
        }
    }
}

#[async_trait]
impl SignalStore for FfiBridgeStore {
    async fn put_identity(&self, address: &str, key: [u8; 32]) -> StoreResult<()> {
        let addr = CString::new(address).unwrap();
        let res = unsafe { (self.ffi.put_identity)(self.ffi.handle, addr.as_ptr(), key.as_ptr(), key.len()) };
        if res == 0 { Ok(()) } else { Err(make_err("put_identity failed")) }
    }

    async fn load_identity(&self, address: &str) -> StoreResult<Option<[u8; 32]>> {
        let addr = CString::new(address).unwrap();
        let mut out = TryxBuffer { data: ptr::null_mut(), len: 0 };
        let res = unsafe { (self.ffi.load_identity)(self.ffi.handle, addr.as_ptr(), &mut out) };
        if res == 0 {
            let vec = out.into_vec();
            if vec.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&vec);
                Ok(Some(arr))
            } else { Err(make_err("invalid identity length")) }
        } else if res == 1 { Ok(None) } else { Err(make_err("load_identity failed")) }
    }

    async fn delete_identity(&self, address: &str) -> StoreResult<()> {
        let addr = CString::new(address).unwrap();
        let res = unsafe { (self.ffi.delete_identity)(self.ffi.handle, addr.as_ptr()) };
        if res == 0 { Ok(()) } else { Err(make_err("delete_identity failed")) }
    }

    async fn get_session(&self, address: &str) -> StoreResult<Option<Bytes>> {
        let addr = CString::new(address).unwrap();
        let mut out = TryxBuffer { data: ptr::null_mut(), len: 0 };
        let res = unsafe { (self.ffi.get_session)(self.ffi.handle, addr.as_ptr(), &mut out) };
        if res == 0 { Ok(Some(Bytes::from(out.into_vec()))) } else if res == 1 { Ok(None) } else { Err(make_err("get_session failed")) }
    }

    async fn put_session(&self, address: &str, session: &[u8]) -> StoreResult<()> {
        let addr = CString::new(address).unwrap();
        let res = unsafe { (self.ffi.put_session)(self.ffi.handle, addr.as_ptr(), session.as_ptr(), session.len()) };
        if res == 0 { Ok(()) } else { Err(make_err("put_session failed")) }
    }

    async fn delete_session(&self, address: &str) -> StoreResult<()> {
        let addr = CString::new(address).unwrap();
        let res = unsafe { (self.ffi.delete_session)(self.ffi.handle, addr.as_ptr()) };
        if res == 0 { Ok(()) } else { Err(make_err("delete_session failed")) }
    }

    async fn store_prekey(&self, id: u32, record: &[u8], uploaded: bool) -> StoreResult<()> {
        let res = unsafe { (self.ffi.store_prekey)(self.ffi.handle, id, record.as_ptr(), record.len(), uploaded as i32) };
        if res == 0 { Ok(()) } else { Err(make_err("store_prekey failed")) }
    }

    async fn load_prekey(&self, id: u32) -> StoreResult<Option<Bytes>> {
        let mut out = TryxBuffer { data: ptr::null_mut(), len: 0 };
        let res = unsafe { (self.ffi.load_prekey)(self.ffi.handle, id, &mut out) };
        if res == 0 { Ok(Some(Bytes::from(out.into_vec()))) } else if res == 1 { Ok(None) } else { Err(make_err("load_prekey failed")) }
    }

    async fn remove_prekey(&self, id: u32) -> StoreResult<()> {
        let res = unsafe { (self.ffi.remove_prekey)(self.ffi.handle, id) };
        if res == 0 { Ok(()) } else { Err(make_err("remove_prekey failed")) }
    }

    async fn get_max_prekey_id(&self) -> StoreResult<u32> {
        let mut out = 0;
        let res = unsafe { (self.ffi.get_max_prekey_id)(self.ffi.handle, &mut out) };
        if res == 0 { Ok(out) } else { Err(make_err("get_max_prekey_id failed")) }
    }

    async fn store_signed_prekey(&self, id: u32, record: &[u8]) -> StoreResult<()> {
        let res = unsafe { (self.ffi.store_signed_prekey)(self.ffi.handle, id, record.as_ptr(), record.len()) };
        if res == 0 { Ok(()) } else { Err(make_err("store_signed_prekey failed")) }
    }

    async fn load_signed_prekey(&self, id: u32) -> StoreResult<Option<Vec<u8>>> {
        let mut out = TryxBuffer { data: ptr::null_mut(), len: 0 };
        let res = unsafe { (self.ffi.load_signed_prekey)(self.ffi.handle, id, &mut out) };
        if res == 0 { Ok(Some(out.into_vec())) } else if res == 1 { Ok(None) } else { Err(make_err("load_signed_prekey failed")) }
    }

    async fn load_all_signed_prekeys(&self) -> StoreResult<Vec<(u32, Vec<u8>)>> {
        Ok(Vec::new()) // Unused by core client except for migration
    }

    async fn remove_signed_prekey(&self, id: u32) -> StoreResult<()> {
        let res = unsafe { (self.ffi.remove_signed_prekey)(self.ffi.handle, id) };
        if res == 0 { Ok(()) } else { Err(make_err("remove_signed_prekey failed")) }
    }

    async fn put_sender_key(&self, address: &str, record: &[u8]) -> StoreResult<()> {
        let addr = CString::new(address).unwrap();
        let res = unsafe { (self.ffi.put_sender_key)(self.ffi.handle, addr.as_ptr(), record.as_ptr(), record.len()) };
        if res == 0 { Ok(()) } else { Err(make_err("put_sender_key failed")) }
    }

    async fn get_sender_key(&self, address: &str) -> StoreResult<Option<Vec<u8>>> {
        let addr = CString::new(address).unwrap();
        let mut out = TryxBuffer { data: ptr::null_mut(), len: 0 };
        let res = unsafe { (self.ffi.get_sender_key)(self.ffi.handle, addr.as_ptr(), &mut out) };
        if res == 0 { Ok(Some(out.into_vec())) } else if res == 1 { Ok(None) } else { Err(make_err("get_sender_key failed")) }
    }

    async fn delete_sender_key(&self, address: &str) -> StoreResult<()> {
        let addr = CString::new(address).unwrap();
        let res = unsafe { (self.ffi.delete_sender_key)(self.ffi.handle, addr.as_ptr()) };
        if res == 0 { Ok(()) } else { Err(make_err("delete_sender_key failed")) }
    }
}

#[async_trait]
impl AppSyncStore for FfiBridgeStore {
    async fn get_sync_key(&self, key_id: &[u8]) -> StoreResult<Option<AppStateSyncKey>> {
        let mut out = TryxBuffer { data: ptr::null_mut(), len: 0 };
        let res = unsafe { (self.ffi.get_sync_key)(self.ffi.handle, key_id.as_ptr(), key_id.len(), &mut out) };
        if res == 0 {
            Ok(Some(bincode::deserialize(&out.into_vec()).map_err(|e| make_err(e.to_string()))?))
        } else if res == 1 { Ok(None) } else { Err(make_err("get_sync_key failed")) }
    }

    async fn set_sync_key(&self, key_id: &[u8], key: AppStateSyncKey) -> StoreResult<()> {
        let data = bincode::serialize(&key).map_err(|e| make_err(e.to_string()))?;
        let res = unsafe { (self.ffi.set_sync_key)(self.ffi.handle, key_id.as_ptr(), key_id.len(), data.as_ptr(), data.len()) };
        if res == 0 { Ok(()) } else { Err(make_err("set_sync_key failed")) }
    }

    async fn get_version(&self, name: &str) -> StoreResult<HashState> {
        let cname = CString::new(name).unwrap();
        let mut out = TryxBuffer { data: ptr::null_mut(), len: 0 };
        let res = unsafe { (self.ffi.get_version)(self.ffi.handle, cname.as_ptr(), &mut out) };
        if res == 0 {
            Ok(bincode::deserialize(&out.into_vec()).unwrap_or_default())
        } else { Ok(HashState::default()) }
    }

    async fn set_version(&self, name: &str, state: HashState) -> StoreResult<()> {
        let cname = CString::new(name).unwrap();
        let data = bincode::serialize(&state).map_err(|e| make_err(e.to_string()))?;
        let res = unsafe { (self.ffi.set_version)(self.ffi.handle, cname.as_ptr(), data.as_ptr(), data.len()) };
        if res == 0 { Ok(()) } else { Err(make_err("set_version failed")) }
    }

    async fn put_mutation_macs(&self, name: &str, version: u64, mutations: &[AppStateMutationMAC]) -> StoreResult<()> {
        let args = serde_json::json!({
            "name": name,
            "version": version,
            "macs": mutations,
        });
        self.call(24, args)?;
        Ok(())
    }

    async fn get_mutation_mac(&self, name: &str, index_mac: &[u8]) -> StoreResult<Option<Vec<u8>>> {
        let args = serde_json::json!({
            "name": name,
            "index_mac": index_mac,
        });
        self.call(25, args)
    }

    async fn delete_mutation_macs(&self, name: &str, index_macs: &[Vec<u8>]) -> StoreResult<()> {
        let args = serde_json::json!({
            "name": name,
            "index_macs": index_macs,
        });
        self.call(26, args)?;
        Ok(())
    }

    async fn get_latest_sync_key_id(&self) -> StoreResult<Option<Vec<u8>>> {
        let mut out = TryxBuffer { data: ptr::null_mut(), len: 0 };
        let res = unsafe { (self.ffi.get_latest_sync_key_id)(self.ffi.handle, &mut out) };
        if res == 0 { Ok(Some(out.into_vec())) } else if res == 1 { Ok(None) } else { Err(make_err("get_latest_sync_key_id failed")) }
    }
}

#[async_trait]
impl DeviceStore for FfiBridgeStore {
    async fn save(&self, device: &wacore::store::Device) -> StoreResult<()> {
        let data = bincode::serialize(device).map_err(|e| make_err(e.to_string()))?;
        let res = unsafe { (self.ffi.save_device)(self.ffi.handle, data.as_ptr(), data.len()) };
        if res == 0 { Ok(()) } else { Err(make_err("save_device failed")) }
    }

    async fn load(&self) -> StoreResult<Option<wacore::store::Device>> {
        let mut out = TryxBuffer { data: ptr::null_mut(), len: 0 };
        let res = unsafe { (self.ffi.load_device)(self.ffi.handle, &mut out) };
        if res == 0 {
            Ok(Some(bincode::deserialize(&out.into_vec()).map_err(|e| make_err(e.to_string()))?))
        } else if res == 1 { Ok(None) } else { Err(make_err("load_device failed")) }
    }

    async fn exists(&self) -> StoreResult<bool> {
        let mut out = 0;
        let res = unsafe { (self.ffi.device_exists)(self.ffi.handle, &mut out) };
        if res == 0 { Ok(out != 0) } else { Err(make_err("device_exists failed")) }
    }

    async fn create(&self) -> StoreResult<i32> {
        let mut out = 0;
        let res = unsafe { (self.ffi.create_device)(self.ffi.handle, &mut out) };
        if res == 0 { Ok(out) } else { Err(make_err("create_device failed")) }
    }
}

#[async_trait]
impl ProtocolStore for FfiBridgeStore {
    async fn get_sender_key_devices(&self, group_jid: &str) -> StoreResult<Vec<(String, bool)>> {
        let args = serde_json::json!({"group_jid": group_jid});
        let bytes = self.call(30, args)?.unwrap_or_default();
        if bytes.is_empty() { Ok(Vec::new()) } else { Ok(serde_json::from_slice(&bytes).unwrap()) }
    }

    async fn set_sender_key_status(&self, group_jid: &str, entries: &[(&str, bool)]) -> StoreResult<()> {
        let args = serde_json::json!({"group_jid": group_jid, "entries": entries});
        self.call(31, args)?;
        Ok(())
    }

    async fn clear_sender_key_devices(&self, group_jid: &str) -> StoreResult<()> {
        let args = serde_json::json!({"group_jid": group_jid});
        self.call(32, args)?;
        Ok(())
    }

    async fn delete_sender_key_device_rows(&self, device_jids: &[&str]) -> StoreResult<()> {
        let args = serde_json::json!({"device_jids": device_jids});
        self.call(34, args)?;
        Ok(())
    }

    async fn clear_all_sender_key_devices(&self) -> StoreResult<()> {
        self.call(33, serde_json::json!({}))?;
        Ok(())
    }

    async fn get_lid_mapping(&self, lid: &str) -> StoreResult<Option<LidPnMappingEntry>> {
        let args = serde_json::json!({"lid": lid});
        let bytes = self.call(35, args)?;
        if let Some(b) = bytes {
            Ok(Some(serde_json::from_slice(&b).unwrap()))
        } else { Ok(None) }
    }

    async fn get_pn_mapping(&self, _phone: &str) -> StoreResult<Option<LidPnMappingEntry>> {
        Ok(None) // Unused by core client except internal
    }

    async fn put_lid_mapping(&self, entry: &LidPnMappingEntry) -> StoreResult<()> {
        let args = serde_json::to_value(entry).unwrap();
        self.call(37, args)?;
        Ok(())
    }

    async fn get_all_lid_mappings(&self) -> StoreResult<Vec<LidPnMappingEntry>> {
        Ok(Vec::new()) // Unused by core client
    }

    async fn save_base_key(&self, _address: &str, _message_id: &str, _base_key: &[u8]) -> StoreResult<()> {
        Ok(()) // Not critical for basic function
    }

    async fn has_same_base_key(&self, _address: &str, _message_id: &str, _current_base_key: &[u8]) -> StoreResult<bool> {
        Ok(false)
    }

    async fn delete_base_key(&self, _address: &str, _message_id: &str) -> StoreResult<()> {
        Ok(())
    }

    async fn update_device_list(&self, _record: DeviceListRecord) -> StoreResult<()> {
        Ok(())
    }

    async fn get_devices(&self, _user: &str) -> StoreResult<Option<DeviceListRecord>> {
        Ok(None)
    }

    async fn delete_devices(&self, _user: &str) -> StoreResult<()> {
        Ok(())
    }

    async fn get_tc_token(&self, _jid: &str) -> StoreResult<Option<TcTokenEntry>> {
        Ok(None)
    }

    async fn put_tc_token(&self, _jid: &str, _entry: &TcTokenEntry) -> StoreResult<()> {
        Ok(())
    }

    async fn delete_tc_token(&self, _jid: &str) -> StoreResult<()> {
        Ok(())
    }

    async fn get_all_tc_token_jids(&self) -> StoreResult<Vec<String>> {
        Ok(Vec::new())
    }

    async fn delete_expired_tc_tokens(&self, _cutoff: i64) -> StoreResult<u32> {
        Ok(0)
    }

    async fn store_sent_message(&self, chat_jid: &str, message_id: &str, payload: &[u8]) -> StoreResult<()> {
        let args = serde_json::json!({
            "chat_jid": chat_jid,
            "message_id": message_id,
            "payload": payload,
        });
        self.call(52, args)?;
        Ok(())
    }

    async fn take_sent_message(&self, chat_jid: &str, message_id: &str) -> StoreResult<Option<Vec<u8>>> {
        let args = serde_json::json!({
            "chat_jid": chat_jid,
            "message_id": message_id,
        });
        self.call(53, args)
    }

    async fn delete_expired_sent_messages(&self, _cutoff: i64) -> StoreResult<u32> {
        Ok(0)
    }
}

#[async_trait]
impl MsgSecretStore for FfiBridgeStore {
    async fn put_msg_secrets(&self, entries: Vec<MsgSecretEntry>) -> StoreResult<usize> {
        let args = serde_json::json!({"entries": entries});
        let bytes = self.call(60, args)?.unwrap_or_default();
        if bytes.is_empty() { Ok(0) } else {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&bytes);
            Ok(u64::from_le_bytes(arr) as usize)
        }
    }

    async fn get_msg_secret(&self, chat: &str, sender: &str, msg_id: &str) -> StoreResult<Option<Vec<u8>>> {
        let args = serde_json::json!({"chat": chat, "sender": sender, "msg_id": msg_id});
        self.call(61, args)
    }

    async fn delete_expired_msg_secrets(&self, cutoff: i64) -> StoreResult<u32> {
        let args = serde_json::json!({"cutoff": cutoff});
        let bytes = self.call(63, args)?.unwrap_or_default();
        if bytes.is_empty() { Ok(0) } else {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(&bytes);
            Ok(u32::from_le_bytes(arr))
        }
    }
}

