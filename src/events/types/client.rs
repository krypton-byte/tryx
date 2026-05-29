#[pyclass]
pub struct DeviceProp {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub manufacturer: String,
    #[pyo3(get)]
    pub model: String,
    #[pyo3(get)]
    pub os_version: String,
}