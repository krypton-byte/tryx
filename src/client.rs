use std::sync::{Arc, Once};
use std::future::Future;
use std::pin::Pin;
use pyo3::{Bound, PyAny, pyclass, pymethods};
use pyo3::prelude::*;
use pyo3_async_runtimes::{TaskLocals, into_future_with_locals};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals, into_future};
use tokio::runtime;
use tokio::sync::watch;
use tokio::time::{Duration, interval};
use wacore::types::events::Event;
use whatsapp_rust::Client;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::store::Backend;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;
use waproto::whatsapp::Message as WhatsappMessage;
use waproto::whatsapp::message::{self as wa};
use prost::Message;
use tokio::signal;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

use crate::backend::{SqliteBackend, BackendBase};
use crate::events::{Connected, LoggedOut, Message as WAMessage, PairingQrCode};
use crate::exceptions::UnsupportedBackend;
use crate::types::{JID, UploadResponse};
use crate::dispatcher::Dispatcher;
use crate::wacore::MediaType;

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
    tryx_client: Py<TryxClient>,
    client_tx: watch::Sender<Option<Arc<Client>>>,
}

#[pyclass]
pub struct TryxClient {
    client_rx: watch::Receiver<Option<Arc<Client>>>,
}

#[pymethods]
impl TryxClient {
    fn is_connected(&self) -> bool {
        self.client_rx.borrow().is_some()
    }
    fn download_media<'py>(&self, py: Python<'py>, message: Py<PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running")
        })?;
        let message_type_name = message
            .getattr(py, "DESCRIPTOR")
            .and_then(|descriptor| descriptor.getattr(py, "name"))
            .and_then(|name| name.extract::<String>(py))
            .unwrap_or_default();
        let serialized: Vec<u8> = message
            .call_method0(py, "SerializeToString")?
            .extract(py)?;

        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            let download = match message_type_name.as_str() {
                "ImageMessage" => {
                    let media = wa::ImageMessage::decode(serialized.as_slice()).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            format!("Failed to decode ImageMessage: {}", e),
                        )
                    })?;
                    client.download(&media).await
                }
                "VideoMessage" => {
                    let media = wa::VideoMessage::decode(serialized.as_slice()).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            format!("Failed to decode VideoMessage: {}", e),
                        )
                    })?;
                    client.download(&media).await
                }
                "DocumentMessage" => {
                    let media = wa::DocumentMessage::decode(serialized.as_slice()).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            format!("Failed to decode DocumentMessage: {}", e),
                        )
                    })?;
                    client.download(&media).await
                }
                "AudioMessage" => {
                    let media = wa::AudioMessage::decode(serialized.as_slice()).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            format!("Failed to decode AudioMessage: {}", e),
                        )
                    })?;
                    client.download(&media).await
                }
                "StickerMessage" => {
                    let media = wa::StickerMessage::decode(serialized.as_slice()).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            format!("Failed to decode StickerMessage: {}", e),
                        )
                    })?;
                    client.download(&media).await
                }
                _ => {
                    // Fallback path for unknown wrappers from Python side.
                    if let Ok(media) = wa::ImageMessage::decode(serialized.as_slice()) {
                        client.download(&media).await
                    } else if let Ok(media) = wa::VideoMessage::decode(serialized.as_slice()) {
                        client.download(&media).await
                    } else if let Ok(media) = wa::DocumentMessage::decode(serialized.as_slice()) {
                        client.download(&media).await
                    } else if let Ok(media) = wa::AudioMessage::decode(serialized.as_slice()) {
                        client.download(&media).await
                    } else if let Ok(media) = wa::StickerMessage::decode(serialized.as_slice()) {
                        client.download(&media).await
                    } else {
                        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "Failed to decode message as supported media message",
                        ));
                    }
                }
            };

            download.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }
    fn upload_file<'py>(&self, py: Python<'py>, path: String, media_type: Py<MediaType>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running")
        })?;
        let media_type_enum = media_type.bind(py).borrow_mut().to_wacore_enum();
        let locals = get_current_locals(py)?;
        let data = std::fs::read(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        future_into_py_with_locals(py, locals, async move {
            let url = client
                .upload(data, media_type_enum)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            let result= UploadResponse {
                url: url.url,
                direct_path: url.direct_path,
                media_key: url.media_key,
                file_enc_sha256: url.file_enc_sha256,
                file_sha256: url.file_sha256,
                file_length: url.file_length,
            };
            Ok(result)
        })
    }
    fn upload<'py>(&self, py: Python<'py>, data: &[u8], media_type: Py<MediaType>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running")
        })?;
        let data_vec = data.to_vec();
        let mtype = media_type.bind(py).borrow_mut().to_wacore_enum();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, UploadResponse>(py, locals, async move {
            let url = client
                .upload(data_vec, mtype)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            let result= UploadResponse {
                url: url.url,
                direct_path: url.direct_path,
                media_key: url.media_key,
                file_enc_sha256: url.file_enc_sha256,
                file_sha256: url.file_sha256,
                file_length: url.file_length,
            };
            Ok(result)
        })
        //     Ok(url)
        // })
    }
    fn send_message<'py>(&self, py: Python<'py>, to: Py<JID>, message: Py<PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running")
        })?;

        let jid = to.bind(py).borrow().as_whatsapp_jid();

        // Python protobuf object -> bytes -> Rust proto
        let serialized: Vec<u8> = message
            .call_method0(py, "SerializeToString")?
            .extract(py)?;

        let whatsapp_message = WhatsappMessage::decode(serialized.as_slice()).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to decode WhatsAppMessage proto: {}", e),
            )
        })?;

        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            let message_id = client
                .send_message(jid, whatsapp_message)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(message_id.to_string())
        })
    }
}

impl Tryx {
    async fn run_bot(
        backend: Arc<dyn Backend>,
        handlers: Py<Dispatcher>,
        locals: Option<TaskLocals>,
        tryx_client: Py<TryxClient>,
        client_tx: watch::Sender<Option<Arc<Client>>>,
    ) -> PyResult<()> {
        let (pairing_qr_callbacks, message_callbacks, connected_callbacks, logout_callbacks) =
            Python::attach(|py| {
                let dispatcher = handlers.bind(py).borrow();
                (
                    dispatcher.pairing_qr_handlers(py),
                    dispatcher.message_handlers(py),
                    dispatcher.conneccted_handlers(py),
                    dispatcher.logout_handlers(py),
                )
            });
        let pairing_qr_callbacks = Arc::new(pairing_qr_callbacks);
        let message_callbacks = Arc::new(message_callbacks);
        let connected_callbacks = Arc::new(connected_callbacks);
        let logout_callbacks = Arc::new(logout_callbacks);
        info!(
            pairing_qr_handlers = pairing_qr_callbacks.len(),
            message_handlers = message_callbacks.len(),
            connected_handlers = connected_callbacks.len(),
            logout_handlers = logout_callbacks.len(),
            "cached dispatcher handlers for runtime"
        );

        info!("building WhatsApp bot");
        let mut bot = Bot::builder()
            .with_backend(backend)
            .with_transport_factory(TokioWebSocketTransportFactory::new())
            .with_http_client(UreqHttpClient::new())
            .on_event(move |event, _client| {
                let locals = locals.clone();
                let pairing_qr_callbacks = Arc::clone(&pairing_qr_callbacks);
                let message_callbacks = Arc::clone(&message_callbacks);
                let connected_callbacks = Arc::clone(&connected_callbacks);
                let logout_callbacks = Arc::clone(&logout_callbacks);
                let tryx_client = Python::attach(|py| tryx_client.clone_ref(py));
                async move {
                    match event {
                        Event::PairingQrCode { code, timeout } => {
                            info!(timeout_secs = timeout.as_secs(), "received pairing QR event");
                            info!(handlers = pairing_qr_callbacks.len(), "dispatching pairing QR handlers");

                            for (idx, callback) in pairing_qr_callbacks.iter().enumerate() {
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
                            info!(handlers = message_callbacks.len(), message_id = %info.id, "dispatching message handlers");

                            for (idx, callback) in message_callbacks.iter().enumerate() {
                                debug!(handler_index = idx, message_id = %info.id, "calling message Python callback");
                                let locals = locals.clone();
                                let py_future = Python::attach(|py| -> PyResult<_> {
                                    let payload = Py::new(py, WAMessage::new(msg.clone(), info.clone()))?;
                                    let client_obj = tryx_client.clone_ref(py);
                                    let awaitable = callback.bind(py).call1((client_obj, payload))?;
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
                        Event::Connected(_) => {
                            for (idx, callback) in connected_callbacks.iter().enumerate() {
                                debug!(handler_index = idx, "calling connected event handler");
                                let _ = Python::attach(|py| -> PyResult<_> {
                                    let awaitable = callback.bind(py).call1((Connected{},))?;
                                    let fut = into_future(awaitable)?;
                                    Ok(fut)
                                });
                            }
                        }
                        Event::LoggedOut(logout) => {
                            for (idx, callback) in logout_callbacks.iter().enumerate() {
                                debug!(handler_index = idx, "calling logged out event handler");
                                let _ = Python::attach(|py| -> PyResult<_> {
                                    let awaitable = callback.bind(py).call1((LoggedOut::new(logout.clone()),))?;
                                    let fut = into_future(awaitable)?;
                                    Ok(fut)
                                });
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

        let client = bot.client();
        client_tx
            .send(Some(client))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

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
            let (client_tx, client_rx) = watch::channel(None);
            let tryx_client = Py::new(py, TryxClient { client_rx })?;
            info!("backend connected and dispatcher initialized");
            Ok(Tryx {
                backend: Arc::new(store),
                handlers: Py::new(py, Dispatcher::empty())?,
                tryx_client,
                client_tx,
            })
        } else {
            error!("unsupported backend type passed to Tryx");
            Err(PyErr::new::<UnsupportedBackend, _>("Unsupported backend type"))
        }
    }
    fn get_client(&self, py: Python<'_>) -> Py<TryxClient> {
        self.tryx_client.clone_ref(py)
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
        let tryx_client = self.tryx_client.clone_ref(py);
        let client_tx = self.client_tx.clone();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals.clone(), async move {
            Self::run_bot(backend, handlers, Some(locals), tryx_client, client_tx).await
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
        let tryx_client = Python::attach(|py| self.tryx_client.clone_ref(py));
        let client_tx = self.client_tx.clone();
        py.detach(move || {
            let rt = runtime::Runtime::new()
                .map_err(|e| {
                    error!(error = %e, "failed to create tokio runtime for blocking mode");
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                })?;

            rt.block_on(async {
                let mut bot_task = tokio::spawn(Self::run_bot(backend, handlers, None, tryx_client, client_tx));
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