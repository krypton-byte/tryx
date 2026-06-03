
use whatsapp_rust_sqlite_storage::SqliteStore as SqliteStoreInner;
use pyo3::prelude::*;
use tracing::{error, info};

#[pyclass(subclass)]
pub struct BackendBase;

#[pyclass(extends=BackendBase)]
pub struct SqliteStore {
    #[pyo3(get)]
    path: String,
}

#[pymethods]
impl SqliteStore {
    #[new]
    fn new(path: String) -> (Self, BackendBase) {
        (SqliteStore { path }, BackendBase)
    }
}

impl SqliteStore {
    pub async fn connect(&self) -> Result<SqliteStoreInner, String> {
        info!(path = %self.path, "connecting sqlite backend");
        match SqliteStoreInner::new(&self.path).await {
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
