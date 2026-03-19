use pyo3::{pyclass, pymethods};
use pyo3::exceptions::PyException;
#[pyclass(extends=PyException)]
pub struct FailedBuildBot {
    message: String,
}

#[pymethods]
impl FailedBuildBot {
    #[new]
    fn new(message: String) -> Self {
        FailedBuildBot { message }
    }
    fn __str__(&self) -> String {
        self.message.clone()
    }
    fn __repr__(&self) -> String {
        format!("FailedBuildBot(message='{}')", self.message)
    }
}

#[pyclass(extends=PyException)]
struct FailedToDecodeProto {
    message: String,
}

#[pymethods]
impl FailedToDecodeProto {
    #[new]
    fn new(message: String) -> Self {
        FailedToDecodeProto { message }
    }
    fn __str__(&self) -> String {
        self.message.clone()
    }
    fn __repr__(&self) -> String {
        format!("FailedToDecodeProto(message='{}')", self.message)
    }
}

#[pyclass(extends=PyException)]
pub struct UnsupportedEventType {
    pub message: String,
}

#[pymethods]
impl UnsupportedEventType {
    #[new]
    fn new(message: String) -> Self {
        UnsupportedEventType { message }
    }
    fn __str__(&self) -> String {
        self.message.clone()
    }
    fn __repr__(&self) -> String {
        format!("UnsupportedEventType(message='{}')", self.message)
    }
}

#[pyclass(extends=PyException)]
pub struct UnsupportedBackend {
    message: String,
}

#[pymethods]
impl UnsupportedBackend {
    #[new]
    fn new(message: String) -> Self {
        UnsupportedBackend { message }
    }
    fn __str__(&self) -> String {
        self.message.clone()
    }
    fn __repr__(&self) -> String {
        format!("UnsupportedBackend(message='{}')", self.message)
    }
}

#[pyclass(extends=PyException)]
pub struct EventDispatchError {
    message: String,
}

#[pymethods]
impl EventDispatchError {
    #[new]
    fn new(message: String) -> Self {
        EventDispatchError { message }
    }
    fn __str__(&self) -> String {
        self.message.clone()
    }
    fn __repr__(&self) -> String {
        format!("EventDispatchError(message='{}')", self.message)
    }
}

#[pyclass(extends=PyException)]
pub struct PyPayloadBuildError {
    message: String,
}

#[pymethods]
impl PyPayloadBuildError {
    #[new]
    fn new(message: String) -> Self {
        PyPayloadBuildError { message }
    }
    fn __str__(&self) -> String {
        self.message.clone()
    }
    fn __repr__(&self) -> String {
        format!("PyPayloadBuildError(message='{}')", self.message)
    }
}
