use pyo3::{pyclass, pymethods};

use crate::wacore::iq::presence::PresenceStatus;

#[pyclass]
pub struct PresenceHelpers;

#[pymethods]
impl PresenceHelpers {
    #[staticmethod]
    fn default_status() -> PresenceStatus {
        PresenceStatus::Available
    }
}
