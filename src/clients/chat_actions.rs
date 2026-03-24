use std::sync::Arc;

use prost::Message;
use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use waproto::whatsapp as wa;
use whatsapp_rust::Client;
use whatsapp_rust::{SyncActionMessageRange, message_key, message_range};

use crate::events::proto_cache::{
    parse_proto_bytes,
    proto_message_key,
    proto_sync_action_message_range,
};
use crate::types::JID;

#[pyclass]
pub struct ChatActionsClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl ChatActionsClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running"))
    }

    fn decode_sync_action_message_range(
        py: Python<'_>,
        message_range: Option<Py<PyAny>>,
    ) -> PyResult<Option<SyncActionMessageRange>> {
        match message_range {
            Some(range) => {
                let serialized: Vec<u8> =
                    range.call_method0(py, "SerializeToString")?.extract(py)?;
                let parsed = SyncActionMessageRange::decode(serialized.as_slice()).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        format!("Failed to decode SyncActionMessageRange: {}", e),
                    )
                })?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    fn decode_message_key(py: Python<'_>, key: Py<PyAny>) -> PyResult<wa::MessageKey> {
        let serialized: Vec<u8> = key.call_method0(py, "SerializeToString")?.extract(py)?;
        wa::MessageKey::decode(serialized.as_slice()).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to decode MessageKey: {}", e),
            )
        })
    }

    fn encode_message_key(py: Python<'_>, key: wa::MessageKey) -> PyResult<Py<PyAny>> {
        let mut bytes = Vec::new();
        key.encode(&mut bytes).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to encode MessageKey: {}", e),
            )
        })?;
        parse_proto_bytes(py, proto_message_key(py)?, bytes.as_slice())
    }

    fn encode_sync_action_message_range(
        py: Python<'_>,
        range: SyncActionMessageRange,
    ) -> PyResult<Py<PyAny>> {
        let mut bytes = Vec::new();
        range.encode(&mut bytes).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to encode SyncActionMessageRange: {}", e),
            )
        })?;
        parse_proto_bytes(py, proto_sync_action_message_range(py)?, bytes.as_slice())
    }
}

#[pymethods]
impl ChatActionsClient {
    #[staticmethod]
    fn build_message_key<'py>(
        py: Python<'py>,
        id: String,
        remote_jid: Py<JID>,
        from_me: bool,
        participant: Option<Py<JID>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let remote = remote_jid.bind(py).borrow().as_whatsapp_jid();
        let participant_jid = participant
            .as_ref()
            .map(|p| p.bind(py).borrow().as_whatsapp_jid());
        let key = message_key(id, &remote, from_me, participant_jid.as_ref());
        Ok(Self::encode_message_key(py, key)?.into_bound(py).into_any())
    }

    #[staticmethod]
    fn build_message_range<'py>(
        py: Python<'py>,
        last_message_timestamp: i64,
        last_system_message_timestamp: Option<i64>,
        messages: Vec<(Py<PyAny>, i64)>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let parsed_messages = messages
            .into_iter()
            .map(|(key, ts)| Self::decode_message_key(py, key).map(|parsed| (parsed, ts)))
            .collect::<PyResult<Vec<_>>>()?;
        let range = message_range(
            last_message_timestamp,
            last_system_message_timestamp,
            parsed_messages,
        );
        Ok(Self::encode_sync_action_message_range(py, range)?
            .into_bound(py)
            .into_any())
    }

    fn archive_chat<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        message_range: Option<Py<PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;
        let parsed_range = Self::decode_sync_action_message_range(py, message_range)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .chat_actions()
                .archive_chat(&jid_value, parsed_range)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn unarchive_chat<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        message_range: Option<Py<PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;
        let parsed_range = Self::decode_sync_action_message_range(py, message_range)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .chat_actions()
                .unarchive_chat(&jid_value, parsed_range)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn pin_chat<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .chat_actions()
                .pin_chat(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn unpin_chat<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .chat_actions()
                .unpin_chat(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn mute_chat<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .chat_actions()
                .mute_chat(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn mute_chat_until<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        mute_end_timestamp_ms: i64,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .chat_actions()
                .mute_chat_until(&jid_value, mute_end_timestamp_ms)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn unmute_chat<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .chat_actions()
                .unmute_chat(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn star_message<'py>(
        &self,
        py: Python<'py>,
        chat_jid: Py<JID>,
        participant_jid: Option<Py<JID>>,
        message_id: String,
        from_me: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let chat_jid_value = chat_jid.bind(py).borrow().as_whatsapp_jid();
        let participant_jid_value = participant_jid
            .as_ref()
            .map(|p| p.bind(py).borrow().as_whatsapp_jid());
        let locals = get_current_locals(py)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .chat_actions()
                .star_message(
                    &chat_jid_value,
                    participant_jid_value.as_ref(),
                    message_id.as_str(),
                    from_me,
                )
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn unstar_message<'py>(
        &self,
        py: Python<'py>,
        chat_jid: Py<JID>,
        participant_jid: Option<Py<JID>>,
        message_id: String,
        from_me: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let chat_jid_value = chat_jid.bind(py).borrow().as_whatsapp_jid();
        let participant_jid_value = participant_jid
            .as_ref()
            .map(|p| p.bind(py).borrow().as_whatsapp_jid());
        let locals = get_current_locals(py)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .chat_actions()
                .unstar_message(
                    &chat_jid_value,
                    participant_jid_value.as_ref(),
                    message_id.as_str(),
                    from_me,
                )
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn mark_chat_as_read<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        read: bool,
        message_range: Option<Py<PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;
        let parsed_range = Self::decode_sync_action_message_range(py, message_range)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .chat_actions()
                .mark_chat_as_read(&jid_value, read, parsed_range)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn delete_chat<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        delete_media: bool,
        message_range: Option<Py<PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;
        let parsed_range = Self::decode_sync_action_message_range(py, message_range)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .chat_actions()
                .delete_chat(&jid_value, delete_media, parsed_range)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn delete_message_for_me<'py>(
        &self,
        py: Python<'py>,
        chat_jid: Py<JID>,
        participant_jid: Option<Py<JID>>,
        message_id: String,
        from_me: bool,
        delete_media: bool,
        message_timestamp: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let chat_jid_value = chat_jid.bind(py).borrow().as_whatsapp_jid();
        let participant_jid_value = participant_jid
            .as_ref()
            .map(|p| p.bind(py).borrow().as_whatsapp_jid());
        let locals = get_current_locals(py)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .chat_actions()
                .delete_message_for_me(
                    &chat_jid_value,
                    participant_jid_value.as_ref(),
                    message_id.as_str(),
                    from_me,
                    delete_media,
                    message_timestamp,
                )
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }
}