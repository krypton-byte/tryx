use std::sync::{Arc, Once};
use std::future::Future;
use std::pin::Pin;
use pyo3::{Bound, PyAny, pyclass, pymethods};
use pyo3::prelude::*;
use pyo3_async_runtimes::{TaskLocals, into_future_with_locals};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals, into_future};
use tokio::runtime;
use tokio::time::{Duration, interval};
use wacore::types::events::Event;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::store::Backend;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;
use tokio::signal;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

use crate::backend::{SqliteBackend, BackendBase};
use crate::events::{Dispatcher, Message, PairingQrCode};

static LOG_INIT: Once = Once::new();

fn init_logging() {
    LOG_INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .with_target(true)
            .try_init();
    });
}

#[pyclass]
pub struct Tryx {
    backend: Arc<dyn Backend>,
    handlers: Py<Dispatcher>,
}

impl Tryx {
    async fn run_bot(
        backend: Arc<dyn Backend>,
        handlers: Py<Dispatcher>,
        locals: Option<TaskLocals>,
    ) -> PyResult<()> {
        info!("building WhatsApp bot");
        let mut bot = Bot::builder()
            .with_backend(backend)
            .with_transport_factory(TokioWebSocketTransportFactory::new())
            .with_http_client(UreqHttpClient::new())
            .on_event(move |event, _client| {
                let handlers = Python::attach(|py| handlers.clone_ref(py));
                let locals = locals.clone();
                async move {
                    match event {
                        Event::PairingQrCode { code, timeout } => {
                            info!(timeout_secs = timeout.as_secs(), "received pairing QR event");
                            let callbacks = Python::attach(|py| {
                                handlers
                                    .bind(py)
                                    .borrow()
                                    .pairing_qr_handlers(py)
                            });
                            info!(handlers = callbacks.len(), "dispatching pairing QR handlers");

                            for (idx, callback) in callbacks.into_iter().enumerate() {
                                debug!(handler_index = idx, "calling pairing QR Python callback");
                                let locals = locals.clone();
                                let py_future = Python::attach(|py| -> PyResult<_> {
                                    let payload = Py::new(py, PairingQrCode::new(code.clone(), timeout.as_secs()))?;
                                    let awaitable = callback.bind(py).call1((py.None(), payload))?;
                                    let fut: Pin<Box<dyn Future<Output = PyResult<Py<PyAny>>> + Send>> = match &locals {
                                        Some(locals) => {
                                            let fut = into_future_with_locals(locals, awaitable)?;
                                            Box::pin(async move { fut.await })
                                        }
                                        None => {
                                            let fut = into_future(awaitable)?;
                                            Box::pin(async move { fut.await })
                                        }
                                    };
                                    Ok(fut)
                                });

                                match py_future {
                                    Ok(py_future) => {
                                        if let Err(err) = py_future.await {
                                            error!(handler_index = idx, error = %err, "pairing QR callback failed");
                                            Python::attach(|py| err.print(py));
                                        } else {
                                            debug!(handler_index = idx, "pairing QR callback finished");
                                        }
                                    }
                                    Err(err) => {
                                        error!(handler_index = idx, error = %err, "failed to schedule pairing QR callback");
                                        Python::attach(|py| err.print(py));
                                    }
                                }
                            }
                        }
                        Event::Message(msg, info) => {
                            debug!(message_id = %info.id, "received message event");
                            let callbacks = Python::attach(|py| {
                                handlers
                                    .bind(py)
                                    .borrow()
                                    .message_handlers(py)
                            });
                            info!(handlers = callbacks.len(), message_id = %info.id, "dispatching message handlers");

                            for (idx, callback) in callbacks.into_iter().enumerate() {
                                debug!(handler_index = idx, message_id = %info.id, "calling message Python callback");
                                let locals = locals.clone();
                                let py_future = Python::attach(|py| -> PyResult<_> {
                                    let payload = Py::new(py, Message::new(msg.clone(), info.clone()))?;
                                    let awaitable = callback.bind(py).call1((py.None(), payload))?;
                                    let fut: Pin<Box<dyn Future<Output = PyResult<Py<PyAny>>> + Send>> = match &locals {
                                        Some(locals) => {
                                            let fut = into_future_with_locals(locals, awaitable)?;
                                            Box::pin(async move { fut.await })
                                        }
                                        None => {
                                            let fut = into_future(awaitable)?;
                                            Box::pin(async move { fut.await })
                                        }
                                    };
                                    Ok(fut)
                                });

                                match py_future {
                                    Ok(py_future) => {
                                        if let Err(err) = py_future.await {
                                            error!(handler_index = idx, message_id = %info.id, error = %err, "message callback failed");
                                            Python::attach(|py| err.print(py));
                                        } else {
                                            debug!(handler_index = idx, message_id = %info.id, "message callback finished");
                                        }
                                    }
                                    Err(err) => {
                                        error!(handler_index = idx, message_id = %info.id, error = %err, "failed to schedule message callback");
                                        Python::attach(|py| err.print(py));
                                    }
                                }
                            }
                        }
                        _ => {
                            debug!("received event without registered dispatcher path");
                        }
                    }
                }
            })
            .build()
            .await
            .map_err(|e| {
                error!(error = %e, "failed to build bot");
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
            })?;

        info!("bot built successfully, starting run loop");

        bot.run()
            .await
            .map_err(|e| {
                error!(error = %e, "failed to start bot run stream");
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
            })?
            .await
            .map_err(|e| {
                error!(error = %e, "bot run stream failed");
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
            })?;

        info!("bot run loop finished");

        Ok(())
    }
}

#[pymethods]
impl Tryx {
    #[new]
    fn new(py: Python, backend: Py<BackendBase>) -> PyResult<Self> {
        init_logging();
        info!("initializing Tryx client");
        if let Ok(sqlite) = backend.extract::<Py<SqliteBackend>>(py) {
            debug!("detected sqlite backend from Python");
            let backends = sqlite.borrow(py);
            let rt = runtime::Runtime::new()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            let store = rt
                .block_on(backends.connect())
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
            info!("backend connected and dispatcher initialized");
            Ok(Tryx {
                backend: Arc::new(store),
                handlers: Py::new(py, Dispatcher::empty())?,
            })
        } else {
            error!("unsupported backend type passed to Tryx");
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Unsupported backend type"))
        }
    }

    /// Returns a decorator compatible with:
    /// @client.on(Message)
    /// async def on_message(client, data): ...
    fn on(&self, py: Python, event_type: &Bound<PyAny>) -> PyResult<Py<PyAny>> {
        debug!("registering event decorator through Tryx.on");
        let decorator = self
            .handlers
            .bind(py)
            .call_method1("on", (event_type,))?;
        Ok(decorator.unbind())
    }

    fn run<'py>(&'py self, py: Python<'py>) -> Result<Bound<'py, PyAny>, PyErr> {
        init_logging();
        info!("starting bot in async mode via Tryx.run");
        let backend = self.backend.clone();
        let handlers = self.handlers.clone_ref(py);
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals.clone(), async move {
            Self::run_bot(backend, handlers, Some(locals)).await
        })
    }

    /// Starts the bot and blocks until it exits.
    ///
    /// Python usage:
    /// client.run_blocking()
    fn run_blocking(&self, py: Python<'_>) -> PyResult<()> {
        init_logging();
        info!("starting bot in blocking mode via Tryx.run_blocking");
        let backend = self.backend.clone();
        let handlers = Python::attach(|py| self.handlers.clone_ref(py));

        py.detach(move || {
            let rt = runtime::Runtime::new()
                .map_err(|e| {
                    error!(error = %e, "failed to create tokio runtime for blocking mode");
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                })?;

            rt.block_on(async {
                let mut bot_task = tokio::spawn(Self::run_bot(backend, handlers, None));
                let mut signal_tick = interval(Duration::from_millis(200));

                loop {
                    tokio::select! {
                        _ = signal::ctrl_c() => {
                            warn!("SIGINT received via tokio::signal, stopping bot task");
                            bot_task.abort();
                            break;
                        }
                        _ = signal_tick.tick() => {
                            let signal_result = Python::attach(|py| py.check_signals());
                            if let Err(err) = signal_result {
                                let is_keyboard_interrupt = Python::attach(|py| err.is_instance_of::<pyo3::exceptions::PyKeyboardInterrupt>(py));
                                if is_keyboard_interrupt {
                                    warn!("KeyboardInterrupt detected from Python, stopping bot task");
                                    bot_task.abort();
                                    break;
                                }

                                error!(error = %err, "non-keyboard Python signal error while polling");
                                bot_task.abort();
                                return Err(err);
                            }
                        }
                        result = &mut bot_task => {
                            match result {
                                Ok(inner) => {
                                    info!("bot task finished in blocking mode");
                                    inner?;
                                }
                                Err(err) if err.is_cancelled() => {
                                    info!("bot task cancelled");
                                }
                                Err(err) => {
                                    error!(error = %err, "bot task join failed");
                                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(err.to_string()));
                                }
                            }
                            info!("blocking run finished");
                            return Ok(());
                        }
                    }
                }

                match bot_task.await {
                    Ok(Ok(())) => info!("bot finished after interrupt"),
                    Ok(Err(err)) => return Err(err),
                    Err(join_err) if join_err.is_cancelled() => info!("bot task cancelled successfully"),
                    Err(join_err) => {
                        error!(error = %join_err, "bot task join failed after interrupt");
                        return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(join_err.to_string()));
                    }
                }

                info!("blocking run interrupted and finished");
                Ok(())
            })
        })
    }
}
