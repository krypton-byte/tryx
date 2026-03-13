use pyo3::prelude::*;
/// A Python module implemented in Rust.
/// 
#[pymodule]
fn tryx(_py: &Bound<'_, PyModule>) -> PyResult<()> {
    _py.add_class::<client::Tryx>()?;
    _py.add_class::<backend::SqliteBackend>()?;
    _py.add_class::<types::MessageInfo>()?;
    Ok(())
}

mod backend;
mod client;
mod events;
mod types;