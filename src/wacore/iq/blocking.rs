use pyo3::prelude::*;
use whatsapp_rust::BlocklistEntry as WaBlocklistEntry;

use crate::types::JID;

#[pyclass]
pub struct BlocklistEntry {
    #[pyo3(get)]
    pub jid: Py<JID>,
    #[pyo3(get)]
    pub timestamp: Option<u64>,
}

impl BlocklistEntry {
    pub fn from_inner(py: Python<'_>, value: WaBlocklistEntry) -> PyResult<Self> {
        Ok(Self {
            jid: Py::new(py, JID::from(value.jid))?,
            timestamp: value.timestamp,
        })
    }
}
