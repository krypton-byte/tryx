use pyo3::{Bound, PyAny, PyResult, Python, pyclass, pymethods};
use prost::Message;

use crate::events::proto_cache::parse_message_proto;

#[pyclass]
pub struct NewsletterHelpers;

#[pymethods]
impl NewsletterHelpers {
    #[staticmethod]
    fn parse_message<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyAny>> {
        Ok(parse_message_proto(py, data)?.into_bound(py).into_any())
    }

    #[staticmethod]
    fn serialize_message(py: Python<'_>, message: pyo3::Py<PyAny>) -> PyResult<Vec<u8>> {
        message.call_method0(py, "SerializeToString")?.extract(py)
    }

    #[staticmethod]
    fn build_text_message<'py>(py: Python<'py>, text: String) -> PyResult<Bound<'py, PyAny>> {
        let message = waproto::whatsapp::Message {
            conversation: Some(text),
            ..Default::default()
        };
        Ok(parse_message_proto(py, message.encode_to_vec().as_slice())?
            .into_bound(py)
            .into_any())
    }
}
