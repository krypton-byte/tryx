
use async_trait::async_trait;
use bytes::Bytes;
use pyo3::prelude::*;
use wacore::appstate::hash::HashState;
use wacore::store::traits::*;
use wacore::store::error::{Result as StoreResult, StoreError};
use wacore_appstate::processor::AppStateMutationMAC;
use wacore::store::traits::TcTokenEntry;
use wacore::store::traits::{LidPnMappingEntry, DeviceListRecord, MsgSecretEntry, AppStateSyncKey};

fn make_err(msg: impl ToString) -> StoreError {
    StoreError::Io(std::io::Error::new(std::io::ErrorKind::Other, msg.to_string()))
}

pub struct PythonStore {
    pub py_obj: Py<crate::backend::BackendBase>,
}

impl Clone for PythonStore {
    fn clone(&self) -> Self {
        Python::attach(|_py| Self {
            py_obj: self.py_obj.clone_ref(_py)
        })
    }
}

impl PythonStore {
    pub fn new(py_obj: Py<crate::backend::BackendBase>) -> Self {
        Self { py_obj }
    }
}

#[async_trait]
impl DeviceStore for PythonStore {
    async fn save(&self, device: &wacore::store::Device) -> StoreResult<()> {
        let device_c = serde_json::to_vec(&device).map_err(|e| make_err(e.to_string()))?;
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("device", pyo3::types::PyBytes::new(_py, &device_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("save", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn load(&self) -> StoreResult<Option<wacore::store::Device>> {
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            let coro = self.py_obj.bind(_py).call_method("load", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { let v = _res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?; serde_json::from_slice(&v).map_err(|e| make_err(e.to_string())).map(Some) }
        })
    }

    async fn exists(&self) -> StoreResult<bool> {
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            let coro = self.py_obj.bind(_py).call_method("exists", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            _res.bind(_py).extract::<bool>().map_err(|e| make_err(e.to_string()))
        })
    }

    async fn create(&self) -> StoreResult<i32> {
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            let coro = self.py_obj.bind(_py).call_method("create", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            _res.bind(_py).extract::<i32>().map_err(|e| make_err(e.to_string()))
        })
    }
}

#[async_trait]
impl ProtocolStore for PythonStore {
    async fn get_sender_key_devices(&self, group_jid: &str) -> StoreResult<Vec<(String, bool)>> {
        let group_jid_c = group_jid.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("group_jid", group_jid_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("get_sender_key_devices", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            let v = _res.bind(_py).extract::<Vec<(String, bool)>>().map_err(|e| make_err(e.to_string()))?; Ok(v)
        })
    }

    async fn set_sender_key_status(&self, group_jid: &str, entries: &[(&str, bool)]) -> StoreResult<()> {
        let group_jid_c = group_jid.to_string();
        let entries_c = serde_json::to_vec(&entries).map_err(|e| make_err(e.to_string()))?;
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("group_jid", group_jid_c).unwrap();
            kwargs.set_item("entries", pyo3::types::PyBytes::new(_py, &entries_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("set_sender_key_status", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn clear_sender_key_devices(&self, group_jid: &str) -> StoreResult<()> {
        let group_jid_c = group_jid.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("group_jid", group_jid_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("clear_sender_key_devices", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn delete_sender_key_device_rows(&self, device_jids: &[&str]) -> StoreResult<()> {
        let device_jids_c = serde_json::to_vec(&device_jids).map_err(|e| make_err(e.to_string()))?;
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("device_jids", pyo3::types::PyBytes::new(_py, &device_jids_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("delete_sender_key_device_rows", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn clear_all_sender_key_devices(&self) -> StoreResult<()> {
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            let coro = self.py_obj.bind(_py).call_method("clear_all_sender_key_devices", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn get_lid_mapping(&self, lid: &str) -> StoreResult<Option<LidPnMappingEntry>> {
        let lid_c = lid.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("lid", lid_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("get_lid_mapping", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { let v = _res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?; serde_json::from_slice(&v).map_err(|e| make_err(e.to_string())).map(Some) }
        })
    }

    async fn get_pn_mapping(&self, phone: &str) -> StoreResult<Option<LidPnMappingEntry>> {
        let phone_c = phone.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("phone", phone_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("get_pn_mapping", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { let v = _res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?; serde_json::from_slice(&v).map_err(|e| make_err(e.to_string())).map(Some) }
        })
    }

    async fn put_lid_mapping(&self, entry: &LidPnMappingEntry) -> StoreResult<()> {
        let entry_c = serde_json::to_vec(&entry).map_err(|e| make_err(e.to_string()))?;
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("entry", pyo3::types::PyBytes::new(_py, &entry_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("put_lid_mapping", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn get_all_lid_mappings(&self) -> StoreResult<Vec<LidPnMappingEntry>> {
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            let coro = self.py_obj.bind(_py).call_method("get_all_lid_mappings", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            let v = _res.bind(_py).extract::<Vec<Vec<u8>>>().map_err(|e| make_err(e.to_string()))?; let mut parsed: Vec<LidPnMappingEntry> = Vec::new(); for item in v { parsed.push(serde_json::from_slice::<LidPnMappingEntry>(item.as_slice()).map_err(|e| make_err(e.to_string()))?); } Ok(parsed)
        })
    }

    async fn save_base_key(&self, address: &str, message_id: &str, base_key: &[u8]) -> StoreResult<()> {
        let address_c = address.to_string();
        let message_id_c = message_id.to_string();
        let base_key_c = base_key.to_vec();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("address", address_c).unwrap();
            kwargs.set_item("message_id", message_id_c).unwrap();
            kwargs.set_item("base_key", pyo3::types::PyBytes::new(_py, &base_key_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("save_base_key", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn has_same_base_key(&self, address: &str, message_id: &str, current_base_key: &[u8]) -> StoreResult<bool> {
        let address_c = address.to_string();
        let message_id_c = message_id.to_string();
        let current_base_key_c = current_base_key.to_vec();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("address", address_c).unwrap();
            kwargs.set_item("message_id", message_id_c).unwrap();
            kwargs.set_item("current_base_key", pyo3::types::PyBytes::new(_py, &current_base_key_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("has_same_base_key", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            _res.bind(_py).extract::<bool>().map_err(|e| make_err(e.to_string()))
        })
    }

    async fn delete_base_key(&self, address: &str, message_id: &str) -> StoreResult<()> {
        let address_c = address.to_string();
        let message_id_c = message_id.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("address", address_c).unwrap();
            kwargs.set_item("message_id", message_id_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("delete_base_key", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn update_device_list(&self, record: DeviceListRecord) -> StoreResult<()> {
        let record_c = serde_json::to_vec(&record).map_err(|e| make_err(e.to_string()))?;
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("record", pyo3::types::PyBytes::new(_py, &record_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("update_device_list", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn get_devices(&self, user: &str) -> StoreResult<Option<DeviceListRecord>> {
        let user_c = user.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("user", user_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("get_devices", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { let v = _res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?; serde_json::from_slice(&v).map_err(|e| make_err(e.to_string())).map(Some) }
        })
    }

    async fn delete_devices(&self, user: &str) -> StoreResult<()> {
        let user_c = user.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("user", user_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("delete_devices", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn get_tc_token(&self, jid: &str) -> StoreResult<Option<TcTokenEntry>> {
        let jid_c = jid.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("jid", jid_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("get_tc_token", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { let v = _res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?; serde_json::from_slice(&v).map_err(|e| make_err(e.to_string())).map(Some) }
        })
    }

    async fn put_tc_token(&self, jid: &str, entry: &TcTokenEntry) -> StoreResult<()> {
        let jid_c = jid.to_string();
        let entry_c = serde_json::to_vec(&entry).map_err(|e| make_err(e.to_string()))?;
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("jid", jid_c).unwrap();
            kwargs.set_item("entry", pyo3::types::PyBytes::new(_py, &entry_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("put_tc_token", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn delete_tc_token(&self, jid: &str) -> StoreResult<()> {
        let jid_c = jid.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("jid", jid_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("delete_tc_token", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn get_all_tc_token_jids(&self) -> StoreResult<Vec<String>> {
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            let coro = self.py_obj.bind(_py).call_method("get_all_tc_token_jids", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            let v = _res.bind(_py).extract::<Vec<String>>().map_err(|e| make_err(e.to_string()))?; Ok(v)
        })
    }

    async fn delete_expired_tc_tokens(&self, cutoff: i64) -> StoreResult<u32> {
        let cutoff_c = cutoff.clone();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("cutoff", cutoff_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("delete_expired_tc_tokens", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            _res.bind(_py).extract::<u32>().map_err(|e| make_err(e.to_string()))
        })
    }

    async fn store_sent_message(&self, chat_jid: &str, message_id: &str, payload: &[u8]) -> StoreResult<()> {
        let chat_jid_c = chat_jid.to_string();
        let message_id_c = message_id.to_string();
        let payload_c = payload.to_vec();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("chat_jid", chat_jid_c).unwrap();
            kwargs.set_item("message_id", message_id_c).unwrap();
            kwargs.set_item("payload", pyo3::types::PyBytes::new(_py, &payload_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("store_sent_message", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn take_sent_message(&self, chat_jid: &str, message_id: &str) -> StoreResult<Option<Vec<u8>>> {
        let chat_jid_c = chat_jid.to_string();
        let message_id_c = message_id.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("chat_jid", chat_jid_c).unwrap();
            kwargs.set_item("message_id", message_id_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("take_sent_message", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { Ok(Some(_res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?)) }
        })
    }

    async fn delete_expired_sent_messages(&self, cutoff: i64) -> StoreResult<u32> {
        let cutoff_c = cutoff.clone();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("cutoff", cutoff_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("delete_expired_sent_messages", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            _res.bind(_py).extract::<u32>().map_err(|e| make_err(e.to_string()))
        })
    }
}

#[async_trait]
impl AppSyncStore for PythonStore {
    async fn get_sync_key(&self, key_id: &[u8]) -> StoreResult<Option<AppStateSyncKey>> {
        let key_id_c = key_id.to_vec();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("key_id", pyo3::types::PyBytes::new(_py, &key_id_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("get_sync_key", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { let v = _res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?; serde_json::from_slice(&v).map_err(|e| make_err(e.to_string())).map(Some) }
        })
    }

    async fn clear_mutation_macs(&self, _name: &str) -> StoreResult<()> {
        Ok(())
    }

    async fn set_sync_key(&self, key_id: &[u8], key: AppStateSyncKey) -> StoreResult<()> {
        let key_id_c = key_id.to_vec();
        let key_c = serde_json::to_vec(&key).map_err(|e| make_err(e.to_string()))?;
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("key_id", pyo3::types::PyBytes::new(_py, &key_id_c)).unwrap();
            kwargs.set_item("key", pyo3::types::PyBytes::new(_py, &key_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("set_sync_key", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn get_version(&self, name: &str) -> StoreResult<HashState> {
        let name_c = name.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("name", name_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("get_version", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            let v = _res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?; serde_json::from_slice(&v).map_err(|e| make_err(e.to_string()))
        })
    }

    async fn set_version(&self, name: &str, state: HashState) -> StoreResult<()> {
        let name_c = name.to_string();
        let state_c = serde_json::to_vec(&state).map_err(|e| make_err(e.to_string()))?;
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("name", name_c).unwrap();
            kwargs.set_item("state", pyo3::types::PyBytes::new(_py, &state_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("set_version", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn put_mutation_macs(&self, name: &str, version: u64, mutations: &[AppStateMutationMAC]) -> StoreResult<()> {
        let name_c = name.to_string();
        let version_c = version.clone();
        let mutations_c = serde_json::to_vec(&mutations).map_err(|e| make_err(e.to_string()))?;
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("name", name_c).unwrap();
            kwargs.set_item("version", version_c).unwrap();
            kwargs.set_item("mutations", pyo3::types::PyBytes::new(_py, &mutations_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("put_mutation_macs", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn get_mutation_mac(&self, name: &str, index_mac: &[u8]) -> StoreResult<Option<Vec<u8>>> {
        let name_c = name.to_string();
        let index_mac_c = index_mac.to_vec();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("name", name_c).unwrap();
            kwargs.set_item("index_mac", pyo3::types::PyBytes::new(_py, &index_mac_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("get_mutation_mac", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { Ok(Some(_res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?)) }
        })
    }

    async fn delete_mutation_macs(&self, name: &str, index_macs: &[Vec<u8>]) -> StoreResult<()> {
        let name_c = name.to_string();
        let index_macs_c = serde_json::to_vec(&index_macs).map_err(|e| make_err(e.to_string()))?;
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("name", name_c).unwrap();
            kwargs.set_item("index_macs", pyo3::types::PyBytes::new(_py, &index_macs_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("delete_mutation_macs", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn get_latest_sync_key_id(&self) -> StoreResult<Option<Vec<u8>>> {
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            let coro = self.py_obj.bind(_py).call_method("get_latest_sync_key_id", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { Ok(Some(_res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?)) }
        })
    }
}

#[async_trait]
impl SignalStore for PythonStore {
    async fn put_identity(&self, address: &str, key: [u8; 32]) -> StoreResult<()> {
        let address_c = address.to_string();
        let key_c = key.to_vec();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("address", address_c).unwrap();
            kwargs.set_item("key", pyo3::types::PyBytes::new(_py, &key_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("put_identity", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn mark_prekeys_uploaded(&self, _ids: &[u32]) -> StoreResult<()> {
        Ok(())
    }

    async fn load_identity(&self, address: &str) -> StoreResult<Option<[u8; 32]>> {
        let address_c = address.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("address", address_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("load_identity", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { let v = _res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?; let mut arr = [0u8; 32]; arr.copy_from_slice(&v); Ok(Some(arr)) }
        })
    }

    async fn delete_identity(&self, address: &str) -> StoreResult<()> {
        let address_c = address.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("address", address_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("delete_identity", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn get_session(&self, address: &str) -> StoreResult<Option<Bytes>> {
        let address_c = address.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("address", address_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("get_session", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { Ok(Some(Bytes::from(_res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?))) }
        })
    }

    async fn put_session(&self, address: &str, session: &[u8]) -> StoreResult<()> {
        let address_c = address.to_string();
        let session_c = session.to_vec();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("address", address_c).unwrap();
            kwargs.set_item("session", pyo3::types::PyBytes::new(_py, &session_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("put_session", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn delete_session(&self, address: &str) -> StoreResult<()> {
        let address_c = address.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("address", address_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("delete_session", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn store_prekey(&self, id: u32, record: &[u8], uploaded: bool) -> StoreResult<()> {
        let id_c = id.clone();
        let record_c = record.to_vec();
        let uploaded_c = uploaded.clone();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("id", id_c).unwrap();
            kwargs.set_item("record", pyo3::types::PyBytes::new(_py, &record_c)).unwrap();
            kwargs.set_item("uploaded", uploaded_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("store_prekey", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn load_prekey(&self, id: u32) -> StoreResult<Option<Bytes>> {
        let id_c = id.clone();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("id", id_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("load_prekey", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { Ok(Some(Bytes::from(_res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?))) }
        })
    }

    async fn remove_prekey(&self, id: u32) -> StoreResult<()> {
        let id_c = id.clone();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("id", id_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("remove_prekey", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn get_max_prekey_id(&self) -> StoreResult<u32> {
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            let coro = self.py_obj.bind(_py).call_method("get_max_prekey_id", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            _res.bind(_py).extract::<u32>().map_err(|e| make_err(e.to_string()))
        })
    }

    async fn store_signed_prekey(&self, id: u32, record: &[u8]) -> StoreResult<()> {
        let id_c = id.clone();
        let record_c = record.to_vec();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("id", id_c).unwrap();
            kwargs.set_item("record", pyo3::types::PyBytes::new(_py, &record_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("store_signed_prekey", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn load_signed_prekey(&self, id: u32) -> StoreResult<Option<Vec<u8>>> {
        let id_c = id.clone();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("id", id_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("load_signed_prekey", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { Ok(Some(_res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?)) }
        })
    }

    async fn load_all_signed_prekeys(&self) -> StoreResult<Vec<(u32, Vec<u8>)>> {
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            let coro = self.py_obj.bind(_py).call_method("load_all_signed_prekeys", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            let v = _res.bind(_py).extract::<Vec<(u32, Vec<u8>)>>().map_err(|e| make_err(e.to_string()))?; Ok(v)
        })
    }

    async fn remove_signed_prekey(&self, id: u32) -> StoreResult<()> {
        let id_c = id.clone();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("id", id_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("remove_signed_prekey", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn put_sender_key(&self, address: &str, record: &[u8]) -> StoreResult<()> {
        let address_c = address.to_string();
        let record_c = record.to_vec();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("address", address_c).unwrap();
            kwargs.set_item("record", pyo3::types::PyBytes::new(_py, &record_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("put_sender_key", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }

    async fn get_sender_key(&self, address: &str) -> StoreResult<Option<Vec<u8>>> {
        let address_c = address.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("address", address_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("get_sender_key", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { Ok(Some(_res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?)) }
        })
    }

    async fn delete_sender_key(&self, address: &str) -> StoreResult<()> {
        let address_c = address.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("address", address_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("delete_sender_key", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            Ok(())
        })
    }
}

#[async_trait]
impl MsgSecretStore for PythonStore {
    async fn put_msg_secrets(&self, entries: Vec<MsgSecretEntry>) -> StoreResult<usize> {
        let entries_c = serde_json::to_vec(&entries).map_err(|e| make_err(e.to_string()))?;
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("entries", pyo3::types::PyBytes::new(_py, &entries_c)).unwrap();
            let coro = self.py_obj.bind(_py).call_method("put_msg_secrets", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            _res.bind(_py).extract::<usize>().map_err(|e| make_err(e.to_string()))
        })
    }

    async fn get_msg_secret(&self, chat: &str, sender: &str, msg_id: &str) -> StoreResult<Option<Vec<u8>>> {
        let chat_c = chat.to_string();
        let sender_c = sender.to_string();
        let msg_id_c = msg_id.to_string();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("chat", chat_c).unwrap();
            kwargs.set_item("sender", sender_c).unwrap();
            kwargs.set_item("msg_id", msg_id_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("get_msg_secret", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            if _res.bind(_py).is_none() { Ok(None) } else { Ok(Some(_res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?)) }
        })
    }

    async fn delete_expired_msg_secrets(&self, cutoff: i64) -> StoreResult<u32> {
        let cutoff_c = cutoff.clone();
        let fut = Python::attach(|_py| {
            let kwargs = pyo3::types::PyDict::new(_py);
            kwargs.set_item("cutoff", cutoff_c).unwrap();
            let coro = self.py_obj.bind(_py).call_method("delete_expired_msg_secrets", (), Some(&kwargs)).map_err(|e| make_err(e.to_string()))?;
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        })?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {
            _res.bind(_py).extract::<u32>().map_err(|e| make_err(e.to_string()))
        })
    }
}

