use std::sync::Arc;

use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3::types::{PyDict, PyDictMethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use whatsapp_rust::{Client, EventCreationParams, EventResponseType};

use crate::types::JID;

#[pyclass]
pub struct EventsClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl EventsClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first."))
    }
}

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, Copy, PartialEq)]
pub enum EventResponse {
    Going,
    NotGoing,
    Maybe,
}

impl From<EventResponse> for EventResponseType {
    fn from(value: EventResponse) -> Self {
        match value {
            EventResponse::Going => EventResponseType::Going,
            EventResponse::NotGoing => EventResponseType::NotGoing,
            EventResponse::Maybe => EventResponseType::Maybe,
        }
    }
}

#[pymethods]
impl EventsClient {
    #[pyo3(signature = (chat_jid, name, start_time=None, end_time=None, description=None, join_link=None, is_scheduled_call=None, extra_guests_allowed=None))]
    fn create<'py>(
        &self,
        py: Python<'py>,
        chat_jid: Py<JID>,
        name: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
        description: Option<String>,
        join_link: Option<String>,
        is_scheduled_call: Option<bool>,
        extra_guests_allowed: Option<bool>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let chat = chat_jid.bind(py).borrow().as_whatsapp_jid();
        let params = EventCreationParams::builder()
            .name(name)
            .maybe_start_time(start_time)
            .maybe_end_time(end_time)
            .maybe_description(description)
            .maybe_join_link(join_link)
            .maybe_is_scheduled_call(is_scheduled_call)
            .maybe_extra_guests_allowed(extra_guests_allowed)
            .build();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, Py<PyDict>>(py, locals, async move {
            let created = client
                .events()
                .create(&chat, params)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let dict = PyDict::new(py);
                dict.set_item("message_id", created.send_result().message_id.clone().into_string())?;
                dict.set_item("to", Py::new(py, JID::from(created.send_result().to.clone()))?)?;
                dict.set_item("message_secret", created.secret().as_bytes().to_vec())?;
                Ok(dict.unbind())
            })
        })
    }

    #[pyo3(signature = (chat_jid, event_message_id, event_creator_jid, message_secret, response, extra_guest_count=None))]
    fn respond<'py>(
        &self,
        py: Python<'py>,
        chat_jid: Py<JID>,
        event_message_id: String,
        event_creator_jid: Py<JID>,
        message_secret: Vec<u8>,
        response: EventResponse,
        extra_guest_count: Option<i32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let chat = chat_jid.bind(py).borrow().as_whatsapp_jid();
        let creator = event_creator_jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, String>(py, locals, async move {
            client
                .events()
                .respond_raw(
                    chat,
                    event_message_id.as_str(),
                    &creator,
                    message_secret.as_slice(),
                    response.into(),
                    extra_guest_count,
                )
                .await
                .map(|result| result.message_id.into_string())
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }
}
