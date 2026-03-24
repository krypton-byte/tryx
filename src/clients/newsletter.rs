use std::sync::Arc;

use prost::Message;
use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use waproto::whatsapp::Message as WhatsappMessage;
use whatsapp_rust::Client;

use crate::types::JID;
use crate::wacore::iq::newsletter::{NewsletterMessage, NewsletterMetadata};

#[pyclass]
pub struct NewsletterClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl NewsletterClient {
    pub fn new(client_rx: watch::Receiver<Option<Arc<Client>>>) -> Self {
        Self { client_rx }
    }

    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running"))
    }
}

#[pymethods]
impl NewsletterClient {
    fn list_subscribed<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;

        future_into_py_with_locals::<_, Vec<Py<NewsletterMetadata>>>(py, locals, async move {
            let result = client
                .newsletter()
                .list_subscribed()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                result
                    .into_iter()
                    .map(|item| {
                        let py_item = NewsletterMetadata::from_inner(py, item)?;
                        Py::new(py, py_item)
                    })
                    .collect::<PyResult<Vec<_>>>()
            })
        })
    }

    fn get_metadata<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        future_into_py_with_locals::<_, Py<NewsletterMetadata>>(py, locals, async move {
            let result = client
                .newsletter()
                .get_metadata(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_item = NewsletterMetadata::from_inner(py, result)?;
                Py::new(py, py_item)
            })
        })
    }

    fn get_metadata_by_invite<'py>(
        &self,
        py: Python<'py>,
        invite_code: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;

        future_into_py_with_locals::<_, Py<NewsletterMetadata>>(py, locals, async move {
            let result = client
                .newsletter()
                .get_metadata_by_invite(invite_code.as_str())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_item = NewsletterMetadata::from_inner(py, result)?;
                Py::new(py, py_item)
            })
        })
    }

    #[pyo3(signature = (name, description=None))]
    fn create<'py>(
        &self,
        py: Python<'py>,
        name: String,
        description: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if name.trim().is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "name cannot be empty",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;

        future_into_py_with_locals::<_, Py<NewsletterMetadata>>(py, locals, async move {
            let result = client
                .newsletter()
                .create(name.as_str(), description.as_deref())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_item = NewsletterMetadata::from_inner(py, result)?;
                Py::new(py, py_item)
            })
        })
    }

    fn join<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, Py<NewsletterMetadata>>(py, locals, async move {
            let result = client
                .newsletter()
                .join(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_item = NewsletterMetadata::from_inner(py, result)?;
                Py::new(py, py_item)
            })
        })
    }

    fn leave<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals(py, locals, async move {
            client
                .newsletter()
                .leave(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    #[pyo3(signature = (jid, name=None, description=None))]
    fn update<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        name: Option<String>,
        description: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if name.is_none() && description.is_none() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "at least one of name or description must be provided",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, Py<NewsletterMetadata>>(py, locals, async move {
            let result = client
                .newsletter()
                .update(&jid_value, name.as_deref(), description.as_deref())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let py_item = NewsletterMetadata::from_inner(py, result)?;
                Py::new(py, py_item)
            })
        })
    }

    fn subscribe_live_updates<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, u64>(py, locals, async move {
            client
                .newsletter()
                .subscribe_live_updates(&jid_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn send_message<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        message: Py<PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let serialized: Vec<u8> = message.call_method0(py, "SerializeToString")?.extract(py)?;
        let message_value = WhatsappMessage::decode(serialized.as_slice()).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to decode WhatsAppMessage proto: {}", e),
            )
        })?;

        future_into_py_with_locals::<_, String>(py, locals, async move {
            client
                .newsletter()
                .send_message(&jid_value, &message_value)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn send_reaction<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        server_id: u64,
        reaction: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals(py, locals, async move {
            client
                .newsletter()
                .send_reaction(&jid_value, server_id, reaction.as_str())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    #[pyo3(signature = (jid, count, before=None))]
    fn get_messages<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        count: u32,
        before: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();

        future_into_py_with_locals::<_, Vec<Py<NewsletterMessage>>>(py, locals, async move {
            let result = client
                .newsletter()
                .get_messages(&jid_value, count, before)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                result
                    .into_iter()
                    .map(|item| {
                        let py_item = NewsletterMessage::from_inner(py, item)?;
                        Py::new(py, py_item)
                    })
                    .collect::<PyResult<Vec<_>>>()
            })
        })
    }
}