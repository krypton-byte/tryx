use pyo3::{pyclass, pymethods};

use crate::wacore::iq::chatstate::ChatStateType;

#[pyclass]
pub struct ChatstateHelpers;

#[pymethods]
impl ChatstateHelpers {
    #[staticmethod]
    fn composing() -> ChatStateType {
        ChatStateType::Composing
    }

    #[staticmethod]
    fn recording() -> ChatStateType {
        ChatStateType::Recording
    }

    #[staticmethod]
    fn paused() -> ChatStateType {
        ChatStateType::Paused
    }
}
