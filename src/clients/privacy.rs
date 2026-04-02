use std::sync::Arc;

use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use whatsapp_rust::Client;

use crate::wacore::iq::privacy::{DisallowedListUpdate, PrivacyCategory, PrivacySetting, PrivacyValue};

#[pyclass]
pub struct PrivacyClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl PrivacyClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first."))
    }
}

#[pymethods]
impl PrivacyClient {
    fn fetch_settings<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;

        future_into_py_with_locals::<_, Vec<Py<PrivacySetting>>>(py, locals, async move {
            let result = client
                .fetch_privacy_settings()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            Python::attach(|py| {
                result
                    .settings
                    .into_iter()
                    .map(|item| Py::new(py, PrivacySetting::from(item)))
                    .collect::<PyResult<Vec<_>>>()
            })
        })
    }

    fn set_setting<'py>(
        &self,
        py: Python<'py>,
        category: PrivacyCategory,
        value: PrivacyValue,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rust_category: wacore::iq::privacy::PrivacyCategory = category.into();
        let rust_value: wacore::iq::privacy::PrivacyValue = value.into();
        if !rust_category.is_valid_value(&rust_value) {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "invalid value for privacy category",
            ));
        }

        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, Option<String>>(py, locals, async move {
            client
                .set_privacy_setting(rust_category, rust_value)
                .await
                .map(|result| result.dhash)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn set_disallowed_list<'py>(
        &self,
        py: Python<'py>,
        category: PrivacyCategory,
        update: Py<DisallowedListUpdate>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rust_category: wacore::iq::privacy::PrivacyCategory = category.into();
        if !rust_category.supports_disallowed_list() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "category does not support disallowed list",
            ));
        }

        let update_value = update.bind(py).borrow().to_rust_update(py);
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, Option<String>>(py, locals, async move {
            client
                .set_privacy_disallowed_list(rust_category, update_value)
                .await
                .map(|result| result.dhash)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn set_default_disappearing_mode<'py>(
        &self,
        py: Python<'py>,
        duration_seconds: u32,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;

        future_into_py_with_locals(py, locals, async move {
            client
                .set_default_disappearing_mode(duration_seconds)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }
}
