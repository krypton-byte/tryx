
use std::sync::Arc;
use async_trait::async_trait;
use whatsapp_rust::store::Backend;
use whatsapp_rust_sqlite_storage::SqliteStore;
use pyo3::prelude::*;

#[pyclass(subclass)]
pub struct BackendBase;

#[pyclass(extends=BackendBase)]
pub struct SqliteBackend {
    path: String,
}

#[pyclass(extends=BackendBase)]
pub struct PostgresBackend {
    connection_string: String,
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

#[pymethods]
impl PostgresBackend {
    #[new]
    fn new(connection_string: String) -> (Self, BackendBase) {
        (PostgresBackend { connection_string }, BackendBase)
    }
}
pub enum BackendType {
    Sqlite(SqliteBackend),
    Postgres(PostgresBackend)
}

