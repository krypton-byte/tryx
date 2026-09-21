
use whatsapp_rust_sqlite_storage::{
    SqliteStore as SqliteStoreInner,
    StoredDeviceSummary as StoredDeviceSummaryInner,
};
use pyo3::prelude::*;
use tracing::{error, info};

use crate::types::JID;

#[pyclass(subclass)]
pub struct BackendBase;

#[pyclass]
pub struct StoredDeviceSummary {
    #[pyo3(get)]
    pub id: i32,
    #[pyo3(get)]
    pub pn: Option<Py<JID>>,
    #[pyo3(get)]
    pub lid: Option<Py<JID>>,
    #[pyo3(get)]
    pub push_name: String,
    #[pyo3(get)]
    pub linked: bool,
}

impl StoredDeviceSummary {
    pub fn from_inner(py: Python<'_>, inner: StoredDeviceSummaryInner) -> PyResult<Self> {
        Ok(Self {
            id: inner.id,
            pn: inner.pn.map(|j| Py::new(py, JID::from(j))).transpose()?,
            lid: inner.lid.map(|j| Py::new(py, JID::from(j))).transpose()?,
            push_name: inner.push_name,
            linked: inner.linked,
        })
    }
}

#[pyclass(extends=BackendBase)]
pub struct SqliteStore {
    #[pyo3(get)]
    path: String,
    #[pyo3(get)]
    device_id: i32,
}

#[pymethods]
impl SqliteStore {
    #[new]
    #[pyo3(signature = (path, device_id=0))]
    fn new(path: String, device_id: i32) -> (Self, BackendBase) {
        (SqliteStore { path, device_id }, BackendBase)
    }

    fn create_new_device<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let path = self.path.clone();
        let locals = pyo3_async_runtimes::tokio::get_current_locals(py)?;
        pyo3_async_runtimes::tokio::future_into_py_with_locals::<_, i32>(py, locals, async move {
            let store = SqliteStoreInner::new(&path)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            store
                .create_new_device()
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    fn list_devices<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let path = self.path.clone();
        let locals = pyo3_async_runtimes::tokio::get_current_locals(py)?;
        pyo3_async_runtimes::tokio::future_into_py_with_locals::<_, Vec<Py<StoredDeviceSummary>>>(
            py,
            locals,
            async move {
                let store = SqliteStoreInner::new(&path)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
                let devices = store
                    .list_devices()
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
                Python::attach(|py| {
                    devices
                        .into_iter()
                        .map(|d| {
                            let summary = StoredDeviceSummary::from_inner(py, d)?;
                            Py::new(py, summary)
                        })
                        .collect::<PyResult<Vec<_>>>()
                })
            },
        )
    }

    fn remove_device<'py>(&self, py: Python<'py>, device_id: i32) -> PyResult<Bound<'py, PyAny>> {
        let path = self.path.clone();
        let locals = pyo3_async_runtimes::tokio::get_current_locals(py)?;
        pyo3_async_runtimes::tokio::future_into_py_with_locals(py, locals, async move {
            let store = SqliteStoreInner::new(&path)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            store
                .remove_device(device_id)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Ok(())
        })
    }

    fn device_exists<'py>(&self, py: Python<'py>, device_id: i32) -> PyResult<Bound<'py, PyAny>> {
        let path = self.path.clone();
        let locals = pyo3_async_runtimes::tokio::get_current_locals(py)?;
        pyo3_async_runtimes::tokio::future_into_py_with_locals::<_, bool>(py, locals, async move {
            let store = SqliteStoreInner::new(&path)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            store
                .device_exists(device_id)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }
}

impl SqliteStore {
    pub async fn connect(&self) -> Result<SqliteStoreInner, String> {
        info!(path = %self.path, device_id = self.device_id, "connecting sqlite backend");
        match SqliteStoreInner::new_for_device(&self.path, self.device_id).await {
            Ok(store) => {
                info!(path = %self.path, device_id = self.device_id, "sqlite backend connected");
                Ok(store)
            }
            Err(e) => {
                error!(path = %self.path, device_id = self.device_id, error = %e, "sqlite backend connection failed");
                Err(e.to_string())
            }
        }
    }
}
