use async_trait::async_trait;
use bytes::Bytes;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use serde::de::DeserializeOwned;
use wacore::appstate::hash::HashState;
use wacore::store::error::{Result as StoreResult, StoreError};
use wacore::store::traits::{
    AppStateSyncKey, AppSyncStore, DeviceListRecord, DeviceStore, LidPnMappingEntry,
    MsgSecretEntry, MsgSecretStore, ProtocolStore, SignalStore, StoredMessageSecret, TcTokenEntry,
};
use wacore_appstate::processor::AppStateMutationMAC;

fn make_err(msg: impl ToString) -> StoreError {
    StoreError::Io(std::io::Error::other(msg.to_string()))
}

fn extract_json_from_py<T: DeserializeOwned>(obj: &Bound<'_, PyAny>) -> StoreResult<T> {
    if let Ok(py_bytes) = obj.cast::<PyBytes>() {
        return serde_json::from_slice(py_bytes.as_bytes()).map_err(make_err);
    }
    let v = obj.extract::<Vec<u8>>().map_err(make_err)?;
    serde_json::from_slice(&v).map_err(make_err)
}

fn extract_json_opt<T: DeserializeOwned>(py: Python<'_>, res: &Py<PyAny>) -> StoreResult<Option<T>> {
    let bound = res.bind(py);
    if bound.is_none() {
        Ok(None)
    } else {
        extract_json_from_py(bound).map(Some)
    }
}

fn extract_bytes_opt(py: Python<'_>, res: &Py<PyAny>) -> StoreResult<Option<Vec<u8>>> {
    let bound = res.bind(py);
    if bound.is_none() {
        return Ok(None);
    }
    if let Ok(py_bytes) = bound.cast::<PyBytes>() {
        return Ok(Some(py_bytes.as_bytes().to_vec()));
    }
    bound.extract::<Vec<u8>>().map(Some).map_err(make_err)
}

pub struct PythonStore {
    pub py_obj: Py<pyo3::PyAny>,
}

impl Clone for PythonStore {
    fn clone(&self) -> Self {
        Python::attach(|py| Self {
            py_obj: self.py_obj.clone_ref(py),
        })
    }
}

impl PythonStore {
    pub fn new(py_obj: Py<pyo3::PyAny>) -> Self {
        Self { py_obj }
    }

    /// Invoke a zero-argument async Python store method without allocating a kwargs dict.
    async fn call0(&self, method: &str) -> StoreResult<Py<PyAny>> {
        let fut = Python::attach(|py| {
            let coro = self.py_obj.bind(py).call_method0(method).map_err(make_err)?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(make_err)
        })?;
        fut.await.map_err(make_err)
    }

    /// Invoke an async Python store method with keyword arguments built under a single GIL hold.
    async fn call_kw<F>(&self, method: &str, fill_kwargs: F) -> StoreResult<Py<PyAny>>
    where
        F: FnOnce(Python<'_>, &Bound<'_, PyDict>) -> PyResult<()>,
    {
        let fut = Python::attach(|py| {
            let kwargs = PyDict::new(py);
            fill_kwargs(py, &kwargs).map_err(make_err)?;
            let coro = self
                .py_obj
                .bind(py)
                .call_method(method, (), Some(&kwargs))
                .map_err(make_err)?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(make_err)
        })?;
        fut.await.map_err(make_err)
    }
}

#[async_trait]
impl DeviceStore for PythonStore {
    async fn save(&self, device: &wacore::store::Device) -> StoreResult<()> {
        let device_c = serde_json::to_vec(&device).map_err(make_err)?;
        self.call_kw("save", |py, kw| kw.set_item("device", PyBytes::new(py, &device_c)))
            .await?;
        Ok(())
    }

    async fn load(&self) -> StoreResult<Option<wacore::store::Device>> {
        let res = self.call0("load").await?;
        Python::attach(|py| extract_json_opt(py, &res))
    }

    async fn exists(&self) -> StoreResult<bool> {
        let res = self.call0("exists").await?;
        Python::attach(|py| res.bind(py).extract::<bool>().map_err(make_err))
    }

    async fn create(&self) -> StoreResult<i32> {
        let res = self.call0("create").await?;
        Python::attach(|py| res.bind(py).extract::<i32>().map_err(make_err))
    }
}

#[async_trait]
impl ProtocolStore for PythonStore {
    async fn get_sender_key_devices(&self, group_jid: &str) -> StoreResult<Vec<(String, bool)>> {
        let res = self
            .call_kw("get_sender_key_devices", |_py, kw| kw.set_item("group_jid", group_jid))
            .await?;
        Python::attach(|py| res.bind(py).extract::<Vec<(String, bool)>>().map_err(make_err))
    }

    async fn set_sender_key_status(&self, group_jid: &str, entries: &[(&str, bool)]) -> StoreResult<()> {
        let entries_c = serde_json::to_vec(&entries).map_err(make_err)?;
        self.call_kw("set_sender_key_status", |py, kw| {
            kw.set_item("group_jid", group_jid)?;
            kw.set_item("entries", PyBytes::new(py, &entries_c))
        })
        .await?;
        Ok(())
    }

    async fn clear_sender_key_devices(&self, group_jid: &str) -> StoreResult<()> {
        self.call_kw("clear_sender_key_devices", |_py, kw| kw.set_item("group_jid", group_jid))
            .await?;
        Ok(())
    }

    async fn delete_sender_key_device_rows(&self, device_jids: &[&str]) -> StoreResult<()> {
        let device_jids_c = serde_json::to_vec(&device_jids).map_err(make_err)?;
        self.call_kw("delete_sender_key_device_rows", |py, kw| {
            kw.set_item("device_jids", PyBytes::new(py, &device_jids_c))
        })
        .await?;
        Ok(())
    }

    async fn clear_all_sender_key_devices(&self) -> StoreResult<()> {
        self.call0("clear_all_sender_key_devices").await?;
        Ok(())
    }

    async fn get_lid_mapping(&self, lid: &str) -> StoreResult<Option<LidPnMappingEntry>> {
        let res = self
            .call_kw("get_lid_mapping", |_py, kw| kw.set_item("lid", lid))
            .await?;
        Python::attach(|py| extract_json_opt(py, &res))
    }

    async fn get_pn_mapping(&self, phone: &str) -> StoreResult<Option<LidPnMappingEntry>> {
        let res = self
            .call_kw("get_pn_mapping", |_py, kw| kw.set_item("phone", phone))
            .await?;
        Python::attach(|py| extract_json_opt(py, &res))
    }

    async fn put_lid_mapping(&self, entry: &LidPnMappingEntry) -> StoreResult<()> {
        let entry_c = serde_json::to_vec(&entry).map_err(make_err)?;
        self.call_kw("put_lid_mapping", |py, kw| {
            kw.set_item("entry", PyBytes::new(py, &entry_c))
        })
        .await?;
        Ok(())
    }

    async fn get_all_lid_mappings(&self) -> StoreResult<Vec<LidPnMappingEntry>> {
        let res = self.call0("get_all_lid_mappings").await?;
        Python::attach(|py| {
            let bound = res.bind(py);
            if let Ok(list) = bound.cast::<PyList>() {
                let mut parsed = Vec::with_capacity(list.len());
                for item in list.iter() {
                    parsed.push(extract_json_from_py::<LidPnMappingEntry>(&item)?);
                }
                return Ok(parsed);
            }
            let v = bound.extract::<Vec<Vec<u8>>>().map_err(make_err)?;
            let mut parsed = Vec::with_capacity(v.len());
            for item in v {
                parsed.push(serde_json::from_slice(&item).map_err(make_err)?);
            }
            Ok(parsed)
        })
    }

    async fn save_base_key(&self, address: &str, message_id: &str, base_key: &[u8]) -> StoreResult<()> {
        self.call_kw("save_base_key", |py, kw| {
            kw.set_item("address", address)?;
            kw.set_item("message_id", message_id)?;
            kw.set_item("base_key", PyBytes::new(py, base_key))
        })
        .await?;
        Ok(())
    }

    async fn has_same_base_key(&self, address: &str, message_id: &str, current_base_key: &[u8]) -> StoreResult<bool> {
        let res = self
            .call_kw("has_same_base_key", |py, kw| {
                kw.set_item("address", address)?;
                kw.set_item("message_id", message_id)?;
                kw.set_item("current_base_key", PyBytes::new(py, current_base_key))
            })
            .await?;
        Python::attach(|py| res.bind(py).extract::<bool>().map_err(make_err))
    }

    async fn delete_base_key(&self, address: &str, message_id: &str) -> StoreResult<()> {
        self.call_kw("delete_base_key", |_py, kw| {
            kw.set_item("address", address)?;
            kw.set_item("message_id", message_id)
        })
        .await?;
        Ok(())
    }

    async fn update_device_list(&self, record: DeviceListRecord) -> StoreResult<()> {
        let record_c = serde_json::to_vec(&record).map_err(make_err)?;
        self.call_kw("update_device_list", |py, kw| {
            kw.set_item("record", PyBytes::new(py, &record_c))
        })
        .await?;
        Ok(())
    }

    async fn get_devices(&self, user: &str) -> StoreResult<Option<DeviceListRecord>> {
        let res = self
            .call_kw("get_devices", |_py, kw| kw.set_item("user", user))
            .await?;
        Python::attach(|py| extract_json_opt(py, &res))
    }

    async fn delete_devices(&self, user: &str) -> StoreResult<()> {
        self.call_kw("delete_devices", |_py, kw| kw.set_item("user", user))
            .await?;
        Ok(())
    }

    async fn get_tc_token(&self, jid: &str) -> StoreResult<Option<TcTokenEntry>> {
        let res = self
            .call_kw("get_tc_token", |_py, kw| kw.set_item("jid", jid))
            .await?;
        Python::attach(|py| extract_json_opt(py, &res))
    }

    async fn put_tc_token(&self, jid: &str, entry: &TcTokenEntry) -> StoreResult<()> {
        let entry_c = serde_json::to_vec(&entry).map_err(make_err)?;
        self.call_kw("put_tc_token", |py, kw| {
            kw.set_item("jid", jid)?;
            kw.set_item("entry", PyBytes::new(py, &entry_c))
        })
        .await?;
        Ok(())
    }

    async fn delete_tc_token(&self, jid: &str) -> StoreResult<()> {
        self.call_kw("delete_tc_token", |_py, kw| kw.set_item("jid", jid))
            .await?;
        Ok(())
    }

    async fn get_all_tc_token_jids(&self) -> StoreResult<Vec<String>> {
        let res = self.call0("get_all_tc_token_jids").await?;
        Python::attach(|py| res.bind(py).extract::<Vec<String>>().map_err(make_err))
    }

    async fn delete_expired_tc_tokens(&self, token_cutoff: i64, sender_cutoff: i64) -> StoreResult<u32> {
        let res = self
            .call_kw("delete_expired_tc_tokens", |_py, kw| {
                kw.set_item("token_cutoff", token_cutoff)?;
                kw.set_item("sender_cutoff", sender_cutoff)
            })
            .await?;
        Python::attach(|py| res.bind(py).extract::<u32>().map_err(make_err))
    }

    async fn touch_tc_token_sender_timestamp(&self, jid: &str, sender_timestamp: i64) -> StoreResult<()> {
        let mut entry = self.get_tc_token(jid).await?.unwrap_or(TcTokenEntry {
            token: Vec::new(),
            token_timestamp: 0,
            sender_timestamp: None,
        });
        entry.sender_timestamp = Some(sender_timestamp);
        self.put_tc_token(jid, &entry).await
    }

    async fn store_received_tc_token(&self, jid: &str, token: &[u8], token_timestamp: i64) -> StoreResult<()> {
        let mut entry = self.get_tc_token(jid).await?.unwrap_or(TcTokenEntry {
            token: Vec::new(),
            token_timestamp: 0,
            sender_timestamp: None,
        });
        if entry.token.is_empty() || token_timestamp >= entry.token_timestamp {
            entry.token = token.to_vec();
            entry.token_timestamp = token_timestamp;
            self.put_tc_token(jid, &entry).await?;
        }
        Ok(())
    }

    async fn store_sent_message(&self, chat_jid: &str, message_id: &str, payload: &[u8]) -> StoreResult<()> {
        self.call_kw("store_sent_message", |py, kw| {
            kw.set_item("chat_jid", chat_jid)?;
            kw.set_item("message_id", message_id)?;
            kw.set_item("payload", PyBytes::new(py, payload))
        })
        .await?;
        Ok(())
    }

    async fn take_sent_message(&self, chat_jid: &str, message_id: &str) -> StoreResult<Option<Vec<u8>>> {
        let res = self
            .call_kw("take_sent_message", |_py, kw| {
                kw.set_item("chat_jid", chat_jid)?;
                kw.set_item("message_id", message_id)
            })
            .await?;
        Python::attach(|py| extract_bytes_opt(py, &res))
    }

    async fn delete_expired_sent_messages(&self, cutoff: i64) -> StoreResult<u32> {
        let res = self
            .call_kw("delete_expired_sent_messages", |_py, kw| kw.set_item("cutoff", cutoff))
            .await?;
        Python::attach(|py| res.bind(py).extract::<u32>().map_err(make_err))
    }
}

#[async_trait]
impl AppSyncStore for PythonStore {
    async fn get_sync_key(&self, key_id: &[u8]) -> StoreResult<Option<AppStateSyncKey>> {
        let res = self
            .call_kw("get_sync_key", |py, kw| kw.set_item("key_id", PyBytes::new(py, key_id)))
            .await?;
        Python::attach(|py| extract_json_opt(py, &res))
    }

    async fn clear_mutation_macs(&self, _name: &str) -> StoreResult<()> {
        Ok(())
    }

    async fn commit_patch(
        &self,
        name: &str,
        state: HashState,
        removed_index_macs: &[Vec<u8>],
        added: &[AppStateMutationMAC],
    ) -> StoreResult<()> {
        if !removed_index_macs.is_empty() {
            self.delete_mutation_macs(name, removed_index_macs).await?;
        }
        if !added.is_empty() {
            self.put_mutation_macs(name, state.version, added).await?;
        }
        self.set_version(name, state).await
    }

    async fn set_sync_key(&self, key_id: &[u8], key: AppStateSyncKey) -> StoreResult<()> {
        let key_c = serde_json::to_vec(&key).map_err(make_err)?;
        self.call_kw("set_sync_key", |py, kw| {
            kw.set_item("key_id", PyBytes::new(py, key_id))?;
            kw.set_item("key", PyBytes::new(py, &key_c))
        })
        .await?;
        Ok(())
    }

    async fn get_version(&self, name: &str) -> StoreResult<Option<HashState>> {
        let res = self
            .call_kw("get_version", |_py, kw| kw.set_item("name", name))
            .await?;
        Python::attach(|py| extract_json_opt(py, &res))
    }

    async fn delete_version(&self, name: &str) -> StoreResult<()> {
        let fut = Python::attach(|py| -> StoreResult<_> {
            let obj = self.py_obj.bind(py);
            if obj.hasattr("delete_version").unwrap_or(false) {
                let kwargs = PyDict::new(py);
                kwargs.set_item("name", name).map_err(make_err)?;
                let coro = obj
                    .call_method("delete_version", (), Some(&kwargs))
                    .map_err(make_err)?;
                Ok(Some(
                    pyo3_async_runtimes::tokio::into_future(coro).map_err(make_err)?,
                ))
            } else {
                Ok(None)
            }
        })?;
        if let Some(f) = fut {
            f.await.map_err(make_err)?;
        }
        Ok(())
    }

    async fn set_version(&self, name: &str, state: HashState) -> StoreResult<()> {
        let state_c = serde_json::to_vec(&state).map_err(make_err)?;
        self.call_kw("set_version", |py, kw| {
            kw.set_item("name", name)?;
            kw.set_item("state", PyBytes::new(py, &state_c))
        })
        .await?;
        Ok(())
    }

    async fn put_mutation_macs(&self, name: &str, version: u64, mutations: &[AppStateMutationMAC]) -> StoreResult<()> {
        let mutations_c = serde_json::to_vec(&mutations).map_err(make_err)?;
        self.call_kw("put_mutation_macs", |py, kw| {
            kw.set_item("name", name)?;
            kw.set_item("version", version)?;
            kw.set_item("mutations", PyBytes::new(py, &mutations_c))
        })
        .await?;
        Ok(())
    }

    async fn get_mutation_mac(&self, name: &str, index_mac: &[u8]) -> StoreResult<Option<Vec<u8>>> {
        let res = self
            .call_kw("get_mutation_mac", |py, kw| {
                kw.set_item("name", name)?;
                kw.set_item("index_mac", PyBytes::new(py, index_mac))
            })
            .await?;
        Python::attach(|py| extract_bytes_opt(py, &res))
    }

    async fn delete_mutation_macs(&self, name: &str, index_macs: &[Vec<u8>]) -> StoreResult<()> {
        let index_macs_c = serde_json::to_vec(&index_macs).map_err(make_err)?;
        self.call_kw("delete_mutation_macs", |py, kw| {
            kw.set_item("name", name)?;
            kw.set_item("index_macs", PyBytes::new(py, &index_macs_c))
        })
        .await?;
        Ok(())
    }

    async fn get_latest_sync_key_id(&self) -> StoreResult<Option<Vec<u8>>> {
        let res = self.call0("get_latest_sync_key_id").await?;
        Python::attach(|py| extract_bytes_opt(py, &res))
    }
}

#[async_trait]
impl SignalStore for PythonStore {
    async fn put_identity(&self, address: &str, key: [u8; 32]) -> StoreResult<()> {
        self.call_kw("put_identity", |py, kw| {
            kw.set_item("address", address)?;
            kw.set_item("key", PyBytes::new(py, &key))
        })
        .await?;
        Ok(())
    }

    async fn mark_prekeys_uploaded(&self, _ids: &[u32]) -> StoreResult<()> {
        Ok(())
    }

    async fn load_identity(&self, address: &str) -> StoreResult<Option<[u8; 32]>> {
        let res = self
            .call_kw("load_identity", |_py, kw| kw.set_item("address", address))
            .await?;
        Python::attach(|py| {
            let bound = res.bind(py);
            if bound.is_none() {
                return Ok(None);
            }
            let slice_res: StoreResult<[u8; 32]> = if let Ok(py_bytes) = bound.cast::<PyBytes>() {
                py_bytes.as_bytes().try_into().map_err(|_| make_err("identity key must be 32 bytes"))
            } else {
                let v = bound.extract::<Vec<u8>>().map_err(make_err)?;
                v.as_slice().try_into().map_err(|_| make_err("identity key must be 32 bytes"))
            };
            slice_res.map(Some)
        })
    }

    async fn delete_identity(&self, address: &str) -> StoreResult<()> {
        self.call_kw("delete_identity", |_py, kw| kw.set_item("address", address))
            .await?;
        Ok(())
    }

    async fn get_session(&self, address: &str) -> StoreResult<Option<Bytes>> {
        let res = self
            .call_kw("get_session", |_py, kw| kw.set_item("address", address))
            .await?;
        Python::attach(|py| extract_bytes_opt(py, &res).map(|opt| opt.map(Bytes::from)))
    }

    async fn put_session(&self, address: &str, session: &[u8]) -> StoreResult<()> {
        self.call_kw("put_session", |py, kw| {
            kw.set_item("address", address)?;
            kw.set_item("session", PyBytes::new(py, session))
        })
        .await?;
        Ok(())
    }

    async fn delete_session(&self, address: &str) -> StoreResult<()> {
        self.call_kw("delete_session", |_py, kw| kw.set_item("address", address))
            .await?;
        Ok(())
    }

    async fn store_prekey(&self, id: u32, record: &[u8], uploaded: bool) -> StoreResult<()> {
        self.call_kw("store_prekey", |py, kw| {
            kw.set_item("id", id)?;
            kw.set_item("record", PyBytes::new(py, record))?;
            kw.set_item("uploaded", uploaded)
        })
        .await?;
        Ok(())
    }

    async fn load_prekey(&self, id: u32) -> StoreResult<Option<Bytes>> {
        let res = self
            .call_kw("load_prekey", |_py, kw| kw.set_item("id", id))
            .await?;
        Python::attach(|py| extract_bytes_opt(py, &res).map(|opt| opt.map(Bytes::from)))
    }

    async fn remove_prekey(&self, id: u32) -> StoreResult<()> {
        self.call_kw("remove_prekey", |_py, kw| kw.set_item("id", id))
            .await?;
        Ok(())
    }

    async fn get_max_prekey_id(&self) -> StoreResult<u32> {
        let res = self.call0("get_max_prekey_id").await?;
        Python::attach(|py| res.bind(py).extract::<u32>().map_err(make_err))
    }

    async fn store_signed_prekey(&self, id: u32, record: &[u8]) -> StoreResult<()> {
        self.call_kw("store_signed_prekey", |py, kw| {
            kw.set_item("id", id)?;
            kw.set_item("record", PyBytes::new(py, record))
        })
        .await?;
        Ok(())
    }

    async fn load_signed_prekey(&self, id: u32) -> StoreResult<Option<Vec<u8>>> {
        let res = self
            .call_kw("load_signed_prekey", |_py, kw| kw.set_item("id", id))
            .await?;
        Python::attach(|py| extract_bytes_opt(py, &res))
    }

    async fn load_all_signed_prekeys(&self) -> StoreResult<Vec<(u32, Vec<u8>)>> {
        let res = self.call0("load_all_signed_prekeys").await?;
        Python::attach(|py| res.bind(py).extract::<Vec<(u32, Vec<u8>)>>().map_err(make_err))
    }

    async fn remove_signed_prekey(&self, id: u32) -> StoreResult<()> {
        self.call_kw("remove_signed_prekey", |_py, kw| kw.set_item("id", id))
            .await?;
        Ok(())
    }

    async fn put_sender_key(&self, address: &str, record: &[u8]) -> StoreResult<()> {
        self.call_kw("put_sender_key", |py, kw| {
            kw.set_item("address", address)?;
            kw.set_item("record", PyBytes::new(py, record))
        })
        .await?;
        Ok(())
    }

    async fn get_sender_key(&self, address: &str) -> StoreResult<Option<Vec<u8>>> {
        let res = self
            .call_kw("get_sender_key", |_py, kw| kw.set_item("address", address))
            .await?;
        Python::attach(|py| extract_bytes_opt(py, &res))
    }

    async fn delete_sender_key(&self, address: &str) -> StoreResult<()> {
        self.call_kw("delete_sender_key", |_py, kw| kw.set_item("address", address))
            .await?;
        Ok(())
    }
}

#[async_trait]
impl MsgSecretStore for PythonStore {
    async fn put_msg_secrets(&self, entries: Vec<MsgSecretEntry>) -> StoreResult<usize> {
        let entries_c = serde_json::to_vec(&entries).map_err(make_err)?;
        let res = self
            .call_kw("put_msg_secrets", |py, kw| {
                kw.set_item("entries", PyBytes::new(py, &entries_c))
            })
            .await?;
        Python::attach(|py| res.bind(py).extract::<usize>().map_err(make_err))
    }

    async fn get_msg_secret(&self, chat: &str, sender: &str, msg_id: &str) -> StoreResult<Option<Vec<u8>>> {
        let res = self
            .call_kw("get_msg_secret", |_py, kw| {
                kw.set_item("chat", chat)?;
                kw.set_item("sender", sender)?;
                kw.set_item("msg_id", msg_id)
            })
            .await?;
        Python::attach(|py| extract_bytes_opt(py, &res))
    }

    async fn get_stored_msg_secret(
        &self,
        chat: &str,
        sender: &str,
        msg_id: &str,
    ) -> StoreResult<Option<StoredMessageSecret>> {
        match self.get_msg_secret(chat, sender, msg_id).await? {
            Some(bytes) => StoredMessageSecret::from_stored_bytes(&bytes, 0).map(Some),
            None => Ok(None),
        }
    }

    async fn delete_expired_msg_secrets(&self, cutoff: i64) -> StoreResult<u32> {
        let res = self
            .call_kw("delete_expired_msg_secrets", |_py, kw| kw.set_item("cutoff", cutoff))
            .await?;
        Python::attach(|py| res.bind(py).extract::<u32>().map_err(make_err))
    }
}
