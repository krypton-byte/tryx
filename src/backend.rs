
use whatsapp_rust_sqlite_storage::SqliteStore;
use pyo3::prelude::*;
use tracing::{error, info};

#[pyclass(subclass)]
pub struct BackendBase;

#[pyclass(extends=BackendBase)]
pub struct SqliteBackend {
    path: String,
}

#[pymethods]
impl SqliteBackend {
    #[new]
    fn new(path: String) -> (Self, BackendBase) {
        (SqliteBackend { path }, BackendBase)
    }
}

impl SqliteBackend {
    pub async fn connect(&self) -> Result<SqliteStore, String> {
        info!(path = %self.path, "connecting sqlite backend");
        match SqliteStore::new(&self.path).await {
            Ok(store) => {
                info!(path = %self.path, "sqlite backend connected");
                Ok(store)
            }
            Err(e) => {
                error!(path = %self.path, error = %e, "sqlite backend connection failed");
                Err(e.to_string().into())
            }
        }
    }
}

