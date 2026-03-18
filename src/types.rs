use std::sync::Arc;

use pyo3::{PyAny, PyErr, PyResult, Python, exceptions::PyRuntimeError, pyclass, pymethods, types::{PyAnyMethods, PyBytes, PyDateTime}};
use whatsapp_rust::{Jid as WhatsAppJID};
use wacore::types::message::{BotEditType, EditAttribute, MessageInfo as WhatsAppMessageInfo, MessageSource as WhatsAppMessageSource, MsgBotInfo as WhatsAppMsgBotInfo};
use prost::Message;


#[pyclass(skip_from_py_object)]
#[derive(Clone)]
pub struct JID {
    inner: Arc<WhatsAppJID>,
}
impl From<WhatsAppJID> for JID {
    fn from(jid: WhatsAppJID) -> Self {
        JID { inner: Arc::new(jid) }
    }
}

impl From<JID> for WhatsAppJID {
    fn from(jid: JID) -> Self {
        (*jid.inner).clone()
    }
}

impl From<Arc<WhatsAppJID>> for JID {
    fn from(jid: Arc<WhatsAppJID>) -> Self {
        JID { inner: jid }
    }
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
    fn __repr__(&self) -> String {
        format!("JID(user='{}', server='{}')", self.inner.user, self.inner.server)
    }
}

#[pyclass]
pub struct MessageSource {
    inner: Arc<WhatsAppMessageSource>,
    chat: Arc<WhatsAppJID>,
    sender: Arc<WhatsAppJID>,
}
#[pymethods]
impl MessageSource {
    #[getter]
    fn chat(&self) -> JID {
        self.chat.clone().into()
    }
    #[getter]
    fn sender(&self) -> JID {
        self.sender.clone().into()
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
    fn __repr__(&self) -> String {
        format!("MessageSource(chat='{}', sender='{}')", self.chat, self.sender)
    }
}
impl From<WhatsAppMessageSource> for MessageSource {
    fn from(source: WhatsAppMessageSource) -> Self {
        MessageSource {
            inner: Arc::new(source.clone()),
            chat: Arc::new(source.chat.clone()),
            sender: Arc::new(source.sender.clone()),
        }
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
    fn edit_sender_timestamp(&self, py: Python) -> PyResult<Option<pyo3::Py<PyDateTime>>> {
        self.inner.edit_sender_timestamp_ms.map(|x| {
            let date = PyDateTime::from_timestamp(py, x.naive_utc().and_utc().timestamp_millis()as f64/1000.0, None).map_err(|_| PyErr::new::<PyRuntimeError, _>("Failed to convert timestamp to datetime"))?;
            Ok(date.into())
        }).transpose()
    }
    fn __repr__(&self, py: Python<'_>) -> String {
        format!("MsgBotInfo(edit_type={:?}, edit_sender_timestamp={:?})", self.edit_type(), self.edit_sender_timestamp(py).unwrap_or(None))
    }
}
#[pyclass]
struct MsgMetaInfo {
    #[pyo3(get)]
    target_id: Option<String>,
    #[pyo3(get)]
    target_sender: Option<JID>,
    #[pyo3(get)]
    deprecated_lid_session: Option<bool>,
    #[pyo3(get)]
    thread_message_id: Option<String>,
    #[pyo3(get)]
    thread_message_sender_jid: Option<JID>,
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

impl From<WhatsAppMessageInfo> for MessageInfo {
    fn from(info: WhatsAppMessageInfo) -> Self {
        MessageInfo {
            inner: Arc::new(info.clone()),
            id: info.id.clone(),
            r#type: info.r#type.clone(),
            push_name: info.push_name.clone(),
        }
    }
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
            thread_message_sender_jid: match self.inner.meta_info.thread_message_sender_jid {
                Some(ref jid) => Some(JID { inner: Arc::new(jid.clone()) }),
                None => None,
            },
        }
    }
    #[getter]
    fn verified_name(&self, py: Python<'_>) -> PyResult<Option<pyo3::Py<PyAny>>> {
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
                Ok(Some(proto_instance.into()))
            }
            None => Ok(None), // Placeholder, as VerifiedNameCertificate is not yet implemented
        }
    }
    #[getter]
    fn device_sent_meta(&self) -> Option<DeviceSentMeta> {
        self.inner.device_sent_meta.as_ref().map(|meta| DeviceSentMeta {
            destination_jid: meta.destination_jid.clone(),
            phash: meta.phash.clone(),
        })
    }
    fn __repr__(&self) -> String {
        format!("MessageInfo(id='{}', type='{}', push_name='{}')", self.id, self.r#type, self.push_name)
    }
}

#[pyclass]
pub struct UploadResponse {
    #[pyo3(get)]
    pub url: String,
    #[pyo3(get)]
    pub direct_path: String,
    #[pyo3(get)]
    pub media_key: Vec<u8>,
    #[pyo3(get)]
    pub file_enc_sha256: Vec<u8>,
    #[pyo3(get)]
    pub file_sha256: Vec<u8>,
    #[pyo3(get)]
    pub file_length: u64,
}
pub enum NodeValueEnum {
    String(String),
    Jid(pyo3::Py<JID>),
}

/// PyClass wrapper untuk NodeValue
#[pyclass]
pub struct NodeValue {
    inner: NodeValueEnum,
}

#[pymethods]
impl NodeValue {
    #[new]
    pub fn new_string(value: String) -> Self {
        Self { inner: NodeValueEnum::String(value) }
    }

    #[staticmethod]
    pub fn jid(value: pyo3::Py<JID>) -> Self {
        Self { inner: NodeValueEnum::Jid(value) }
    }

    pub fn set_string(&mut self, value: String) {
        self.inner = NodeValueEnum::String(value);
    }

    pub fn set_jid(&mut self, value: pyo3::Py<JID>) {
        self.inner = NodeValueEnum::Jid(value);
    }

    pub fn __repr__(&self, py: Python<'_>) -> String {
        match &self.inner {
            NodeValueEnum::String(s) => format!("NodeValue::String({})", s),
            NodeValueEnum::Jid(jid) => {
                let jid_ref = jid.bind(py).borrow();
                format!("NodeValue::Jid({})", jid_ref.__repr__())
            }
        }
    }
}

/// NodeContent enum internal
pub enum NodeContentEnum {
    Bytes(Vec<u8>),
    String(String),
    Nodes(Vec<pyo3::Py<Node>>),
}

/// PyClass wrapper untuk NodeContent
#[pyclass]
pub struct NodeContent {
    inner: NodeContentEnum,
}

#[pymethods]
impl NodeContent {
    #[staticmethod]
    pub fn bytes(data: Vec<u8>) -> Self {
        Self { inner: NodeContentEnum::Bytes(data) }
    }

    #[staticmethod]
    pub fn string(data: String) -> Self {
        Self { inner: NodeContentEnum::String(data) }
    }

    #[staticmethod]
    pub fn nodes(data: Vec<pyo3::Py<Node>>) -> Self {
        Self { inner: NodeContentEnum::Nodes(data) }
    }

    pub fn is_bytes(&self) -> bool {
        matches!(self.inner, NodeContentEnum::Bytes(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self.inner, NodeContentEnum::String(_))
    }

    pub fn is_nodes(&self) -> bool {
        matches!(self.inner, NodeContentEnum::Nodes(_))
    }

    pub fn as_bytes<'py>(&self, py: Python<'py>) -> Option<pyo3::Py<PyBytes>> {
        match &self.inner {
            NodeContentEnum::Bytes(b) => Some(PyBytes::new(py, b).unbind()),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match &self.inner {
            NodeContentEnum::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_nodes(&self) -> Option<&Vec<pyo3::Py<Node>>> {
        match &self.inner {
            NodeContentEnum::Nodes(n) => Some(n),
            _ => None,
        }
    }

    pub fn __repr__(&self) -> String {
        match &self.inner {
            NodeContentEnum::Bytes(_) => "NodeContent::Bytes(...)".to_string(),
            NodeContentEnum::String(s) => format!("NodeContent::String({})", s),
            NodeContentEnum::Nodes(n) => format!("NodeContent::Nodes(len={})", n.len()),
        }
    }
}

/// PyClass Node
#[pyclass]
pub struct Node {
    #[pyo3(get, set)]
    pub tag: String,

    #[pyo3(get, set)]
    pub content: Option<pyo3::Py<NodeContent>>,
}

#[pymethods]
impl Node {
    #[new]
    pub fn new(tag: String, content: Option<pyo3::Py<NodeContent>>) -> Self {
        Self { tag, content }
    }

    pub fn __repr__(&self, py: Python<'_>) -> String {
        match &self.content {
            Some(c) => {
                let content_ref = c.bind(py).borrow();
                format!("Node(tag={}, content={})", self.tag, content_ref.__repr__())
            }
            None => format!("Node(tag={}, content=None)", self.tag),
        }
    }
}