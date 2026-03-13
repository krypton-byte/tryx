
use whatsapp_rust_sqlite_storage::SqliteStore;
use pyo3::prelude::*;

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
        match SqliteStore::new(&self.path).await {
            Ok(store) => Ok(store),
            Err(e) => Err(e.to_string().into()),
        }
    }
}

