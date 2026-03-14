use pyo3::prelude::*;

use crate::client::{Tryx};
/// A Python module implemented in Rust.
/// 
#[pymodule]
fn _tryx(_py: &Bound<PyModule>) -> PyResult<()> {
    // m.
    let client_module = PyModule::new(_py.py(), "client")?;
    client_module.add_class::<client::TryxClient>()?;
    client_module.add_class::<Tryx>()?;
    _py.add_submodule(&client_module)?;

    let events_module = PyModule::new(_py.py(), "events")?;
    events_module.add_class::<events::Message>()?;
    events_module.add_class::<events::PairingQrCode>()?;
    _py.add_submodule(&events_module)?;

    let backend_module = PyModule::new(_py.py(), "backend")?;
    backend_module.add_class::<backend::SqliteBackend>()?;
    _py.add_submodule(&backend_module)?;

    let types_module = PyModule::new(_py.py(), "types")?;
    types_module.add_class::<types::JID>()?;
    types_module.add_class::<types::MessageInfo>()?;
    types_module.add_class::<types::UploadResponse>()?;
    _py.add_submodule(&types_module)?;
    let wacore_module = PyModule::new(_py.py(), "wacore")?;
    wacore_module.add_class::<wacore::MediaType>()?;
    _py.add_submodule(&wacore_module)?;
    Ok(())
}

mod backend;
mod client;
mod events;
mod types;
mod dispatcher;
mod exceptions;
mod wacore;