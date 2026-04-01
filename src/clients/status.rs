use std::sync::Arc;

use prost::Message;
use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use waproto::whatsapp::Message as WhatsappMessage;
use whatsapp_rust::Client;

use crate::types::{JID, UploadResponse};
use crate::wacore::iq::status::{StatusPrivacySetting, StatusSendOptions};

fn as_32_bytes(field: &str, value: &[u8]) -> PyResult<[u8; 32]> {
    value.try_into().map_err(|_| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("{field} must contain exactly 32 bytes"),
        )
    })
}

#[pyclass]
pub struct StatusClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl StatusClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running"))
    }
}

#[pymethods]
impl StatusClient {
    #[pyo3(signature = (text, background_argb, font, recipients, options=None))]
    fn send_text<'py>(
        &self,
        py: Python<'py>,
        text: String,
        background_argb: u32,
        font: i32,
        recipients: Vec<Py<JID>>,
        options: Option<Py<StatusSendOptions>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if recipients.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "recipients cannot be empty",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let recipient_values = recipients
            .iter()
            .map(|item| item.bind(py).borrow().as_whatsapp_jid())
            .collect::<Vec<_>>();
        let options_value = options
            .as_ref()
            .map(|v| v.bind(py).borrow().to_rust_options())
            .unwrap_or_default();

        future_into_py_with_locals::<_, String>(py, locals, async move {
            client
                .status()
                .send_text(
                    text.as_str(),
                    background_argb,
                    font,
                    recipient_values.as_slice(),
                    options_value,
                )
                .await
                .map(|result| result.message_id)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    #[pyo3(signature = (upload, thumbnail, recipients, caption=None, options=None))]
    fn send_image<'py>(
        &self,
        py: Python<'py>,
        upload: Py<UploadResponse>,
        thumbnail: &[u8],
        recipients: Vec<Py<JID>>,
        caption: Option<String>,
        options: Option<Py<StatusSendOptions>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if recipients.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "recipients cannot be empty",
            ));
        }

        let upload_ref = upload.bind(py).borrow();
        let media_key = as_32_bytes("upload.media_key", &upload_ref.media_key)?;
        let file_enc_sha256 = as_32_bytes("upload.file_enc_sha256", &upload_ref.file_enc_sha256)?;
        let file_sha256 = as_32_bytes("upload.file_sha256", &upload_ref.file_sha256)?;
        let upload_value = whatsapp_rust::upload::UploadResponse {
            url: upload_ref.url.clone(),
            direct_path: upload_ref.direct_path.clone(),
            media_key,
            file_enc_sha256,
            file_sha256,
            file_length: upload_ref.file_length,
            media_key_timestamp: wacore::time::now_secs(),
        };

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let thumbnail_value = thumbnail.to_vec();
        let recipient_values = recipients
            .iter()
            .map(|item| item.bind(py).borrow().as_whatsapp_jid())
            .collect::<Vec<_>>();
        let options_value = options
            .as_ref()
            .map(|v| v.bind(py).borrow().to_rust_options())
            .unwrap_or_default();

        future_into_py_with_locals::<_, String>(py, locals, async move {
            client
                .status()
                .send_image(
                    upload_value,
                    thumbnail_value,
                    caption.as_deref(),
                    recipient_values.as_slice(),
                    options_value,
                )
                .await
                .map(|result| result.message_id)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    #[pyo3(signature = (upload, thumbnail, duration_seconds, recipients, caption=None, options=None))]
    fn send_video<'py>(
        &self,
        py: Python<'py>,
        upload: Py<UploadResponse>,
        thumbnail: &[u8],
        duration_seconds: u32,
        recipients: Vec<Py<JID>>,
        caption: Option<String>,
        options: Option<Py<StatusSendOptions>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if recipients.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "recipients cannot be empty",
            ));
        }

        let upload_ref = upload.bind(py).borrow();
        let media_key = as_32_bytes("upload.media_key", &upload_ref.media_key)?;
        let file_enc_sha256 = as_32_bytes("upload.file_enc_sha256", &upload_ref.file_enc_sha256)?;
        let file_sha256 = as_32_bytes("upload.file_sha256", &upload_ref.file_sha256)?;
        let upload_value = whatsapp_rust::upload::UploadResponse {
            url: upload_ref.url.clone(),
            direct_path: upload_ref.direct_path.clone(),
            media_key,
            file_enc_sha256,
            file_sha256,
            file_length: upload_ref.file_length,
            media_key_timestamp: wacore::time::now_secs(),
        };

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let thumbnail_value = thumbnail.to_vec();
        let recipient_values = recipients
            .iter()
            .map(|item| item.bind(py).borrow().as_whatsapp_jid())
            .collect::<Vec<_>>();
        let options_value = options
            .as_ref()
            .map(|v| v.bind(py).borrow().to_rust_options())
            .unwrap_or_default();

        future_into_py_with_locals::<_, String>(py, locals, async move {
            client
                .status()
                .send_video(
                    upload_value,
                    thumbnail_value,
                    duration_seconds,
                    caption.as_deref(),
                    recipient_values.as_slice(),
                    options_value,
                )
                .await
                .map(|result| result.message_id)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    #[pyo3(signature = (message, recipients, options=None))]
    fn send_raw<'py>(
        &self,
        py: Python<'py>,
        message: Py<PyAny>,
        recipients: Vec<Py<JID>>,
        options: Option<Py<StatusSendOptions>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if recipients.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "recipients cannot be empty",
            ));
        }

        let serialized: Vec<u8> = message.call_method0(py, "SerializeToString")?.extract(py)?;
        let message_value = WhatsappMessage::decode(serialized.as_slice()).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to decode WhatsAppMessage proto: {}", e),
            )
        })?;

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let recipient_values = recipients
            .iter()
            .map(|item| item.bind(py).borrow().as_whatsapp_jid())
            .collect::<Vec<_>>();
        let options_value = options
            .as_ref()
            .map(|v| v.bind(py).borrow().to_rust_options())
            .unwrap_or_default();

        future_into_py_with_locals::<_, String>(py, locals, async move {
            client
                .status()
                .send_raw(message_value, recipient_values.as_slice(), options_value)
                .await
                .map(|result| result.message_id)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    #[pyo3(signature = (message_id, recipients, options=None))]
    fn revoke<'py>(
        &self,
        py: Python<'py>,
        message_id: String,
        recipients: Vec<Py<JID>>,
        options: Option<Py<StatusSendOptions>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if recipients.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "recipients cannot be empty",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let recipient_values = recipients
            .iter()
            .map(|item| item.bind(py).borrow().as_whatsapp_jid())
            .collect::<Vec<_>>();
        let options_value = options
            .as_ref()
            .map(|v| v.bind(py).borrow().to_rust_options())
            .unwrap_or_default();

        future_into_py_with_locals::<_, String>(py, locals, async move {
            client
                .status()
                .revoke(message_id, recipient_values.as_slice(), options_value)
                .await
                .map(|result| result.message_id)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    #[staticmethod]
    fn default_privacy() -> StatusPrivacySetting {
        StatusPrivacySetting::Contacts
    }
}

