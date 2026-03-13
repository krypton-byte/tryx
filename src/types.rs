use std::sync::Arc;

use pyo3::{PyAny, PyErr, PyResult, Python, exceptions::{PyException, PyRuntimeError}, ffi::PyObject, pyclass, pymethods, types::{PyAnyMethods, PyBytes, PyDateTime, PyType}};
use waproto::whatsapp::VerifiedNameCertificate;
use whatsapp_rust::{Jid as WhatsAppJID};
use wacore::types::message::{BotEditType, EditAttribute, MessageInfo as WhatsAppMessageInfo, MessageSource as WhatsAppMessageSource, MsgBotInfo as WhatsAppMsgBotInfo, MsgMetaInfo as WhatsappMsgMetaInfo};
use prost::Message;


#[pyclass(skip_from_py_object)]
#[derive(Clone)]
pub struct JID {
    inner: Arc<WhatsAppJID>,
}

impl JID {
    pub fn as_whatsapp_jid(&self) -> WhatsAppJID {
        (*self.inner).clone()
    }
}

#[pymethods]
impl JID {
    #[new]
    fn new(user: String, server: String) -> PyResult<Self> {
        let inner = WhatsAppJID::new(&user, &server);
        Ok(JID { inner: Arc::new(inner) })
    }
    #[getter]
    fn user(&self) -> String {
        self.inner.user.clone()
    }
    #[getter]
    fn server(&self) -> String {
        self.inner.server.clone()
    }
}

#[pyclass]
struct MessageSource {
    inner: Arc<WhatsAppMessageSource>,
    chat: Arc<WhatsAppJID>,
    sender: Arc<WhatsAppJID>,
}
#[pymethods]
impl MessageSource {
    #[getter]
    fn chat(&self) -> JID {
        JID { inner: self.chat.clone() }
    }
    #[getter]
    fn sender(&self) -> JID {
        JID { inner: self.sender.clone() }
    }
    #[getter]
    fn is_from_me(&self) -> bool {
        self.inner.is_from_me
    }
    #[getter]
    fn is_group(&self) -> bool {
        self.inner.is_group
    }
    #[getter]
    fn addressing_mode(&self) -> Option<&str> {
        match &self.inner.addressing_mode {
            Some(mode) => {
                match mode {
                    whatsapp_rust::types::message::AddressingMode::Pn => Some("pn"),
                    whatsapp_rust::types::message::AddressingMode::Lid => Some("lid"),
                }
            },
            None => None,
        }
    }
    #[getter]
    fn sender_alt(&self) -> Option<JID> {
        self.inner.sender_alt.as_ref().map(|jid| JID { inner: Arc::new(jid.clone()) })
    }
    #[getter]
    fn recipient_alt(&self) -> Option<JID> {
        self.inner.recipient_alt.as_ref().map(|jid| JID { inner: Arc::new(jid.clone()) })
    }
    #[getter]
    fn broadcast_list_owner(&self) -> Option<JID> {
        self.inner.broadcast_list_owner.as_ref().map(|jid| JID { inner: Arc::new(jid.clone()) })
    }
    #[getter]
    fn recipient(&self) -> Option<JID> {
        self.inner.recipient.as_ref().map(|jid| JID { inner: Arc::new(jid.clone()) })
    }

}



#[pyclass]
struct MsgBotInfo {
    inner: Arc<WhatsAppMsgBotInfo>,
    #[pyo3(get)]
    edit_target_id: Option<String>,
}
#[pymethods]
impl MsgBotInfo {
    #[getter]
    fn edit_type(&self) -> Option<&str> {
        self.inner.edit_type.as_ref().map(|edit_type| match edit_type {
            BotEditType::First => "First",
            BotEditType::Inner => "Inner",
            BotEditType::Last => "Last",
        })
    }
    #[getter]
    fn edit_target_id(&self) -> Option<&str> {
        self.inner.edit_target_id.as_ref().map(|s| s.as_str())
    }
    #[getter]
    fn edit_sender_timestamp_ms(&self, py: Python) -> PyResult<Option<pyo3::Py<PyDateTime>>> {
        match self.inner.edit_sender_timestamp_ms {
            Some(ts) => {
                let date = PyDateTime::from_timestamp(py, ts.timestamp_millis() as f64, None).map_err(|_| PyErr::new::<PyRuntimeError, _>("Failed to convert timestamp to datetime"))?;
                Ok(Some(date.into()))
            },
            None => Ok(None),
        }
    }
}
#[pyclass]
struct MsgMetaInfo {
    inner: Arc<WhatsappMsgMetaInfo>,
    #[pyo3(get)]
    target_id: Option<String>,
    target_sender: Option<JID>,
    #[pyo3(get)]
    deprecated_lid_session: Option<bool>,
    #[pyo3(get)]
    thread_message_id: Option<String>,
}

#[pymethods]
impl MsgMetaInfo {
    #[getter]
    fn target_id(&self) -> Option<&str> {
        self.inner.target_id.as_ref().map(|s| s.as_str())
    }
    #[getter]
    fn target_sender(&self) -> Option<JID> {
        self.inner.target_sender.as_ref().map(|jid| JID { inner: Arc::new(jid.clone()) })
    }
    #[getter]
    fn deprecated_lid_session(&self) -> Option<bool> {
        self.inner.deprecated_lid_session
    }
    #[getter]
    fn thread_message_id(&self) -> Option<&str> {
        self.inner.thread_message_id.as_ref().map(|s| s.as_str())
    }
    #[getter]
    fn thread_message_sender_jid(&self) -> Option<JID> {
        self.inner.thread_message_sender_jid.as_ref().map(|jid| JID { inner: Arc::new(jid.clone()) })
    }
}

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
pub struct MessageInfo {
    pub inner: Arc<WhatsAppMessageInfo>,
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub r#type: String,
    #[pyo3(get)]
    pub push_name: String,
}

#[pyclass]
struct DeviceSentMeta {
    #[pyo3(get)]
    destination_jid: String,
    #[pyo3(get)]
    phash: String,
}

#[pymethods]
impl MessageInfo {
    #[getter]
    fn source(&self) -> MessageSource {
        MessageSource {
            inner: Arc::new(self.inner.source.clone()),
            chat: Arc::new(self.inner.source.chat.clone()),
            sender: Arc::new(self.inner.source.sender.clone()),
        }
    }
    #[getter]
    fn multicast(&self) -> bool {
        self.inner.multicast
    }
    #[getter]
    fn server_id(&self) -> &i32 {
        &self.inner.server_id
    }
    #[getter]
    fn timestamp(&self, py: Python) -> PyResult<pyo3::Py<PyDateTime>> {
        let timestamp = self.inner.timestamp;
        let date = PyDateTime::from_timestamp(py, timestamp.timestamp() as f64, None).map_err(|_| PyErr::new::<PyRuntimeError, _>("Failed to convert timestamp to datetime"))?;
        Ok(date.into())
    }
    #[getter]
    fn media_type(&self) -> &str {
        &self.inner.media_type
    }
    #[getter]
    fn edit(&self) -> &str {
        match self.inner.edit {
            EditAttribute::AdminEdit => "AdminEdit",
            EditAttribute::AdminRevoke => "AdminRevoke",
            EditAttribute::MessageEdit => "MessageEdit",
            EditAttribute::PinInChat => "PinInChat",
            EditAttribute::SenderRevoke => "SenderRevoke",
            EditAttribute::Empty => "Empty",
            EditAttribute::Unknown(_) => "Unknown",
        }
    }
    #[getter]
    fn bot_info(&self) -> Option<MsgBotInfo> {
        match &self.inner.bot_info {
            Some(msg) => {
                Some(MsgBotInfo { inner: Arc::new(msg.clone()), edit_target_id: match msg.edit_target_id {
                        Some(ref s) => Some(s.clone()),
                        None => None,

                } })
            },
            None => None,
        }
    }
    #[getter]
    fn meta_info(&self) -> MsgMetaInfo{
        MsgMetaInfo {
            inner: Arc::new(self.inner.meta_info.clone()),
            target_id: match self.inner.meta_info.target_id {
                Some(ref s) => Some(s.clone()),
                None => None,
            },
            target_sender: match self.inner.meta_info.target_sender {
                Some(ref jid) => Some(JID { inner: Arc::new(jid.clone()) }),
                None => None,
            },
            deprecated_lid_session: self.inner.meta_info.deprecated_lid_session,
            thread_message_id: match self.inner.meta_info.thread_message_id {
                Some(ref s) => Some(s.clone()),
                None => None,
            },
        }
    }
    #[getter]
    fn verified_name(&self, py: Python<'_>) -> PyResult<pyo3::Py<PyAny>> {
        match self.inner.verified_name {
            Some(ref name) => {
                let mut buffer = Vec::new();
                name.encode(&mut buffer).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        format!("Failed to encode VerifiedNameCertificate: {}", e),
                    )
                })?;

                let verified_proto = py.import("waproto.whatsapp_pb2")?;
                let proto_type = verified_proto.getattr("attr_name")?;
                let proto_instance = proto_type.call0()?;
                proto_instance.call_method1("ParseFromString", (PyBytes::new(py, &buffer),))?;
                Ok(proto_instance.into())
            }
            None => Ok(py.None()), // Placeholder, as VerifiedNameCertificate is not yet implemented
        }
    }
    #[getter]
    fn device_sent_meta(&self) -> Option<DeviceSentMeta> {
        self.inner.device_sent_meta.as_ref().map(|meta| DeviceSentMeta {
            destination_jid: meta.destination_jid.clone(),
            phash: meta.phash.clone(),
        })
    }
}
