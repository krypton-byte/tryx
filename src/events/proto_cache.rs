use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::{PyBytes, PyType};

static WHATSAPP_MESSAGE_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static SYNC_ACTION_VALUE_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static LAZY_CONVERSATION_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static CONTACT_UPDATE_ACTION_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static MUTE_ACTION_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static MARK_CHAT_AS_READ_UPDATE_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static HISTORY_SYNC_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static DELETE_CHAT_ACTION_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static DELETE_MESSAGE_FOR_ME_ACTION_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static MESSAGE_KEY_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();
static SYNC_ACTION_MESSAGE_RANGE_PROTO: PyOnceLock<Py<PyType>> = PyOnceLock::new();

fn import_proto_type(py: Python<'_>, attr: &str) -> PyResult<Py<PyType>> {
    let module = py.import("tryx.waproto.whatsapp_pb2")?;
    let message_type = module.getattr(attr)?.cast_into::<PyType>()?;
    Ok(message_type.unbind())
}

fn cached_proto_type(
    py: Python<'_>,
    cache: &'static PyOnceLock<Py<PyType>>,
    attr: &'static str,
) -> PyResult<&'static Py<PyType>> {
    cache.get_or_try_init(py, || import_proto_type(py, attr))
}

pub fn proto_whatsapp_message(py: Python<'_>) -> PyResult<&'static Py<PyType>> {
    cached_proto_type(py, &WHATSAPP_MESSAGE_PROTO, "Message")
}

pub fn proto_sync_action_value(py: Python<'_>) -> PyResult<&'static Py<PyType>> {
    cached_proto_type(py, &SYNC_ACTION_VALUE_PROTO, "SyncActionValue")
}

pub fn proto_lazy_conversation(py: Python<'_>) -> PyResult<&'static Py<PyType>> {
    cached_proto_type(py, &LAZY_CONVERSATION_PROTO, "Conversation")
}

pub fn proto_contact_action(py: Python<'_>) -> PyResult<&'static Py<PyType>> {
    cached_proto_type(py, &CONTACT_UPDATE_ACTION_PROTO, "ContactAction")
}

pub fn proto_mute_action(py: Python<'_>) -> PyResult<&'static Py<PyType>> {
    cached_proto_type(py, &MUTE_ACTION_PROTO, "MuteAction")
}

pub fn proto_mark_chat_as_read_action(py: Python<'_>) -> PyResult<&'static Py<PyType>> {
    cached_proto_type(py, &MARK_CHAT_AS_READ_UPDATE_PROTO, "MarkChatAsReadAction")
}

pub fn proto_history_sync(py: Python<'_>) -> PyResult<&'static Py<PyType>> {
    cached_proto_type(py, &HISTORY_SYNC_PROTO, "HistorySync")
}

pub fn proto_delete_chat_action(py: Python<'_>) -> PyResult<&'static Py<PyType>> {
    cached_proto_type(py, &DELETE_CHAT_ACTION_PROTO, "DeleteChatAction")
}

pub fn proto_delete_message_for_me_action(py: Python<'_>) -> PyResult<&'static Py<PyType>> {
    cached_proto_type(py, &DELETE_MESSAGE_FOR_ME_ACTION_PROTO, "DeleteMessageForMeAction")
}

pub fn proto_message_key(py: Python<'_>) -> PyResult<&'static Py<PyType>> {
    cached_proto_type(py, &MESSAGE_KEY_PROTO, "MessageKey")
}

pub fn proto_sync_action_message_range(py: Python<'_>) -> PyResult<&'static Py<PyType>> {
    cached_proto_type(py, &SYNC_ACTION_MESSAGE_RANGE_PROTO, "SyncActionMessageRange")
}

pub fn parse_proto_bytes(
    py: Python<'_>,
    proto_class: &Py<PyType>,
    proto_bytes: &[u8],
) -> PyResult<Py<PyAny>> {
    let proto_instance = proto_class.bind(py).call0()?;
    proto_instance.call_method1("ParseFromString", (PyBytes::new(py, proto_bytes),))?;
    Ok(proto_instance.into_any().unbind())
}

pub fn parse_message_proto(py: Python<'_>, proto_bytes: &[u8]) -> PyResult<Py<PyAny>> {
    let proto_type = proto_whatsapp_message(py)?;
    parse_proto_bytes(py, proto_type, proto_bytes)
}
