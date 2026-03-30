use std::sync::Arc;

use pyo3::{Bound, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use whatsapp_rust::Client;

#[pyclass]
pub struct ProfileClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl ProfileClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running"))
    }
}

#[pymethods]
impl ProfileClient {
    fn set_push_name<'py>(&self, py: Python<'py>, name: String) -> PyResult<Bound<'py, PyAny>> {
        if name.trim().is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "name cannot be empty",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            client
                .profile()
                .set_push_name(name.as_str())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn set_status_text<'py>(&self, py: Python<'py>, text: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            client
                .profile()
                .set_status_text(text.as_str())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn set_profile_picture<'py>(&self, py: Python<'py>, image_data: &[u8]) -> PyResult<Bound<'py, PyAny>> {
        if image_data.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "image_data cannot be empty",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let data = image_data.to_vec();

        future_into_py_with_locals::<_, String>(py, locals, async move {
            client
                .profile()
                .set_profile_picture(data)
                .await
                .map(|result| result.id)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn remove_profile_picture<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;

        future_into_py_with_locals::<_, String>(py, locals, async move {
            client
                .profile()
                .remove_profile_picture()
                .await
                .map(|result| result.id)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }
}