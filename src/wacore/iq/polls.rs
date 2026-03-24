use pyo3::prelude::*;
use whatsapp_rust::features::PollOptionResult as WaPollOptionResult;

#[pyclass]
pub struct PollOptionResult {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub voters: Vec<String>,
}

impl From<WaPollOptionResult> for PollOptionResult {
    fn from(value: WaPollOptionResult) -> Self {
        Self {
            name: value.name,
            voters: value.voters,
        }
    }
}
