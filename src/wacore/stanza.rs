use pyo3::pyclass;

#[pyclass]
pub struct KeyIndexInfo {
    #[pyo3(get)]
    timestamp: i64,
    #[pyo3(get)]
    signed_bytes: Option<Vec<u8>>,
}
impl KeyIndexInfo {
    pub fn new(timestamp: i64, signed_bytes: Option<Vec<u8>>) -> Self {
        Self { timestamp, signed_bytes }
    }
}