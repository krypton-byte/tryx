use pyo3::{Py, Python, pyclass, types::PyDateTime};
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

#[pyclass]
pub struct BusinessSubscription {
    #[pyo3(get)]
    id: String,
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    expiration_date: Option<Py<PyDateTime>>,
    #[pyo3(get)]
    creation_time: Option<Py<PyDateTime>>,
}

impl From<wacore::stanza::BusinessSubscription> for BusinessSubscription {
    fn from(sub: wacore::stanza::BusinessSubscription) -> Self {
        Python::attach(|py|{
            let expiration_date = sub.expiration_date.map(|d| PyDateTime::from_timestamp(py, d as f64, None).unwrap().unbind());
            let creation_time = sub.creation_time.map(|d|  PyDateTime::from_timestamp(py, d as f64, None).unwrap().unbind());
            BusinessSubscription {
                id: sub.id,
                status: sub.status,
                expiration_date: expiration_date,
                creation_time: creation_time,
            }
        })
    }
}