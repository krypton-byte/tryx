
use std::sync::Arc;
use tokio::sync::OnceCell;
use whatsapp_rust_sqlite_storage::{
    SqliteDatabase,
    SqliteDatabaseConfig,
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
    db_cache: Arc<OnceCell<SqliteDatabase>>,
}

impl SqliteStore {
    async fn open_db<'a>(path: &str, cache: &'a OnceCell<SqliteDatabase>) -> Result<&'a SqliteDatabase, String> {
        cache
            .get_or_try_init(|| async {
                SqliteDatabase::open(path, SqliteDatabaseConfig::default())
                    .await
                    .map_err(|e| e.to_string())
            })
            .await
    }

    pub fn connect_future(&self) -> impl std::future::Future<Output = Result<SqliteStoreInner, String>> + Send + 'static {
        let path = self.path.clone();
        let device_id = self.device_id;
        let cache = Arc::clone(&self.db_cache);
        async move {
            Self::connect_with(path, device_id, cache).await
        }
    }

    async fn connect_with(
        path: String,
        device_id: i32,
        cache: Arc<OnceCell<SqliteDatabase>>,
    ) -> Result<SqliteStoreInner, String> {
        info!(path = %path, device_id = device_id, "connecting sqlite backend");
        let db = Self::open_db(&path, &cache)
            .await
            .map_err(|e| {
                error!(path = %path, device_id = device_id, error = %e, "sqlite backend connection failed");
                e
            })?;
        let store = if db.device_exists(device_id).await.map_err(|e| e.to_string())? {
            db.store(device_id)
        } else {
            db.provision_device(device_id).await.map_err(|e| e.to_string())?
        };
        info!(path = %path, device_id = device_id, "sqlite backend connected");
        Ok(store)
    }
}

#[pymethods]
impl SqliteStore {
    #[new]
    #[pyo3(signature = (path, device_id=0))]
    fn new(path: String, device_id: i32) -> (Self, BackendBase) {
        (
            SqliteStore {
                path,
                device_id,
                db_cache: Arc::new(OnceCell::new()),
            },
            BackendBase,
        )
    }

    fn create_new_device<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let path = self.path.clone();
        let cache = Arc::clone(&self.db_cache);
        let locals = pyo3_async_runtimes::tokio::get_current_locals(py)?;
        pyo3_async_runtimes::tokio::future_into_py_with_locals::<_, i32>(py, locals, async move {
            let db = Self::open_db(&path, &cache)
                .await
                .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
            let store = db
                .create_device()
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Ok(store.device_id())
        })
    }

    fn list_devices<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let path = self.path.clone();
        let cache = Arc::clone(&self.db_cache);
        let locals = pyo3_async_runtimes::tokio::get_current_locals(py)?;
        pyo3_async_runtimes::tokio::future_into_py_with_locals::<_, Vec<Py<StoredDeviceSummary>>>(
            py,
            locals,
            async move {
                let db = Self::open_db(&path, &cache)
                    .await
                    .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
                let devices = db
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
        let cache = Arc::clone(&self.db_cache);
        let locals = pyo3_async_runtimes::tokio::get_current_locals(py)?;
        pyo3_async_runtimes::tokio::future_into_py_with_locals(py, locals, async move {
            let db = Self::open_db(&path, &cache)
                .await
                .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
            db
                .remove_device(device_id)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
            Ok(())
        })
    }

    fn device_exists<'py>(&self, py: Python<'py>, device_id: i32) -> PyResult<Bound<'py, PyAny>> {
        let path = self.path.clone();
        let cache = Arc::clone(&self.db_cache);
        let locals = pyo3_async_runtimes::tokio::get_current_locals(py)?;
        pyo3_async_runtimes::tokio::future_into_py_with_locals::<_, bool>(py, locals, async move {
            let db = Self::open_db(&path, &cache)
                .await
                .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
            db
                .device_exists(device_id)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }
}

impl SqliteStore {
    pub async fn connect(&self) -> Result<SqliteStoreInner, String> {
        Self::connect_with(self.path.clone(), self.device_id, Arc::clone(&self.db_cache)).await
    }
}
