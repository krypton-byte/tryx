use pyo3::{Bound, PyResult, Python, pyclass, pymethods};

use crate::wacore::iq::status::{StatusPrivacySetting, StatusSendOptions};

#[pyclass]
pub struct StatusHelpers;

#[pymethods]
impl StatusHelpers {
    #[staticmethod]
    #[pyo3(signature = (privacy=StatusPrivacySetting::Contacts))]
    fn build_send_options<'py>(
        py: Python<'py>,
        privacy: StatusPrivacySetting,
    ) -> PyResult<Bound<'py, StatusSendOptions>> {
        pyo3::Py::new(py, StatusSendOptions { privacy }).map(|obj| obj.into_bound(py))
    }

    #[staticmethod]
    fn default_privacy() -> StatusPrivacySetting {
        StatusPrivacySetting::Contacts
    }
}
