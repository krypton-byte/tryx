use std::any::Any;
use std::{ sync::Arc};
use pyo3::exceptions::PyException;
use pyo3::{pyclass, pymethods};
use pyo3::prelude::*;
use pyo3_async_runtimes::async_std::future_into_py;
use tokio::runtime;
use wacore::types::events::Event;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::store::Backend;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;

use crate::backend::{BackendType, SqliteBackend, BackendBase};

#[pyclass]
struct KaratClient {
    backend: Arc<dyn Backend>,
}

#[pymethods]
impl KaratClient {
    #[new]
    fn new(py: Python, backend: Py<BackendBase>) -> PyResult<Self> {
        if let Ok(sqlite) = backend.extract::<Py<SqliteBackend>>(py) {
            let backends = sqlite.borrow(py);
            let store = runtime::Runtime::new().unwrap().block_on(backends.connect()).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
            // let store = sqlite_backend.connect().await.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
            Ok(KaratClient { backend: Arc::new(store)})
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Unsupported backend type"))
        }
    }
    fn run<'py>(&'py self, py: Python<'py> ) -> Result<Bound<PyAny>, PyErr> {
        // Here you would implement the logic to run the bot using the backend
        let backend = self.backend.clone();
        future_into_py(py, async move {
            // Example: self.backend.do_something().await;
            let bot = Bot::builder()
                .with_backend(backend)
                .with_transport_factory(TokioWebSocketTransportFactory::new())
                .with_http_client(UreqHttpClient::new())
                .on_event(|event, client| async move {
                    match event {
                        Event::PairingQrCode { code, .. } => println!("QR:\n{}", code),
                        Event::Message(msg, info) => {
                            println!("Message from {}: {:?}", info.source.sender, msg);
                        }
                        _ => {}
                    }
                })
                .build();
            bot.await.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?.run().await. .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?.await.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }
}
