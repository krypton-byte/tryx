use std::sync::{Arc};
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
use waproto::whatsapp::sync_action_value::ArchiveChatAction;
use whatsapp_rust::Client;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::store::Backend;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;
use tokio::signal;
use tracing::{debug, error, info, warn};
use super::tryx_client::TryxClient;
use crate::log::init_logging;
use crate::backend::{SqliteBackend, BackendBase};
use crate::events::types::{EvArchiveUpdate, EvConnected, EvLoggedOut, EvMessage, EvPairingQrCode};
use crate::exceptions::UnsupportedBackend;
use crate::events::dispatcher::Dispatcher;
use crate::types::JID;


#[pyclass]
pub struct Tryx {
    backend: Arc<dyn Backend>,
    handlers: Py<Dispatcher>,
    tryx_client: Py<TryxClient>,
    client_tx: watch::Sender<Option<Arc<Client>>>,
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
    

impl Tryx {
    async fn run_bot(
        backend: Arc<dyn Backend>,
        handlers: Py<Dispatcher>,
        locals: Option<TaskLocals>,
        tryx_client: Py<TryxClient>,
        client_tx: watch::Sender<Option<Arc<Client>>>,
    ) -> PyResult<()> {
        let (
            pairing_qr_callbacks,
            message_callbacks,
            connected_callbacks,
            logout_callbacks,
            receipt_callbacks,
            undecryptable_message_callbacks,
            notification_callbacks,
            chat_presence_callbacks,
            presence_callbacks,
            picture_update_callbacks,
            user_about_update_callbacks,
            joined_group_callbacks,
            group_info_update_callbacks,
            contact_update_callbacks,
            push_name_update_callbacks,
            self_push_name_updated_callbacks,
            pin_update_callbacks,
            mute_update_callbacks,
            archive_update_callbacks,
            mark_chat_as_read_update_callbacks,
            history_sync_callbacks,
            offline_sync_preview_callbacks,
            offline_sync_completed_callbacks,
            device_list_update_callbacks,
            business_status_update_callbacks,
            stream_replaced_callbacks,
            temporary_ban_callbacks,
            connect_failure_callbacks,
            stream_error_callbacks,
        ) = Python::attach(|py| {
            let dispatcher = handlers.bind(py).borrow();
            (
                dispatcher.pairing_qr_handlers(py),
                dispatcher.message_handlers(py),
                dispatcher.connected_handlers(py),
                dispatcher.logout_handlers(py),
                dispatcher.receipt_handlers(py),
                dispatcher.undecryptable_message_handlers(py),
                dispatcher.notification_handlers(py),
                dispatcher.chat_presence_handlers(py),
                dispatcher.presence_handlers(py),
                dispatcher.picture_update_handlers(py),
                dispatcher.user_about_update_handlers(py),
                dispatcher.joined_group_handlers(py),
                dispatcher.group_info_update_handlers(py),
                dispatcher.contact_update_handlers(py),
                dispatcher.push_name_update_handlers(py),
                dispatcher.self_push_name_updated_handlers(py),
                dispatcher.pin_update_handlers(py),
                dispatcher.mute_update_handlers(py),
                dispatcher.archive_update_handlers(py),
                dispatcher.mark_chat_as_read_update_handlers(py),
                dispatcher.history_sync_handlers(py),
                dispatcher.offline_sync_preview_handlers(py),
                dispatcher.offline_sync_completed_handlers(py),
                dispatcher.device_list_update_handlers(py),
                dispatcher.business_status_update_handlers(py),
                dispatcher.stream_replaced_handlers(py),
                dispatcher.temporary_ban_handlers(py),
                dispatcher.connect_failure_handlers(py),
                dispatcher.stream_error_handlers(py),
            )
        });
        let pairing_qr_callbacks = Arc::new(pairing_qr_callbacks);
        let message_callbacks = Arc::new(message_callbacks);
        let connected_callbacks = Arc::new(connected_callbacks);
        let logout_callbacks = Arc::new(logout_callbacks);
        let receipt_callbacks = Arc::new(receipt_callbacks);
        let undecryptable_message_callbacks = Arc::new(undecryptable_message_callbacks);
        let notification_callbacks = Arc::new(notification_callbacks);
        let chat_presence_callbacks = Arc::new(chat_presence_callbacks);
        let presence_callbacks = Arc::new(presence_callbacks);
        let picture_update_callbacks = Arc::new(picture_update_callbacks);
        let user_about_update_callbacks = Arc::new(user_about_update_callbacks);
        let joined_group_callbacks = Arc::new(joined_group_callbacks);
        let group_info_update_callbacks = Arc::new(group_info_update_callbacks);
        let contact_update_callbacks = Arc::new(contact_update_callbacks);
        let push_name_update_callbacks = Arc::new(push_name_update_callbacks);
        let self_push_name_updated_callbacks = Arc::new(self_push_name_updated_callbacks);
        let pin_update_callbacks = Arc::new(pin_update_callbacks);
        let mute_update_callbacks = Arc::new(mute_update_callbacks);
        let archive_update_callbacks = Arc::new(archive_update_callbacks);
        let mark_chat_as_read_update_callbacks = Arc::new(mark_chat_as_read_update_callbacks);
        let history_sync_callbacks = Arc::new(history_sync_callbacks);
        let offline_sync_preview_callbacks = Arc::new(offline_sync_preview_callbacks);
        let offline_sync_completed_callbacks = Arc::new(offline_sync_completed_callbacks);
        let device_list_update_callbacks = Arc::new(device_list_update_callbacks);
        let business_status_update_callbacks = Arc::new(business_status_update_callbacks);
        let stream_replaced_callbacks = Arc::new(stream_replaced_callbacks);
        let temporary_ban_callbacks = Arc::new(temporary_ban_callbacks);
        let connect_failure_callbacks = Arc::new(connect_failure_callbacks);
        let stream_error_callbacks = Arc::new(stream_error_callbacks);

        info!(
            pairing_qr_handlers = pairing_qr_callbacks.len(),
            message_handlers = message_callbacks.len(),
            connected_handlers = connected_callbacks.len(),
            logout_handlers = logout_callbacks.len(),
            receipt_handlers = receipt_callbacks.len(),
            undecryptable_message_handlers = undecryptable_message_callbacks.len(),
            notification_handlers = notification_callbacks.len(),
            chat_presence_handlers = chat_presence_callbacks.len(),
            presence_handlers = presence_callbacks.len(),
            picture_update_handlers = picture_update_callbacks.len(),
            user_about_update_handlers = user_about_update_callbacks.len(),
            joined_group_handlers = joined_group_callbacks.len(),
            group_info_update_handlers = group_info_update_callbacks.len(),
            contact_update_handlers = contact_update_callbacks.len(),
            push_name_update_handlers = push_name_update_callbacks.len(),
            self_push_name_updated_handlers = self_push_name_updated_callbacks.len(),
            pin_update_handlers = pin_update_callbacks.len(),
            mute_update_handlers = mute_update_callbacks.len(),
            archive_update_handlers = archive_update_callbacks.len(),
            mark_chat_as_read_update_handlers = mark_chat_as_read_update_callbacks.len(),
            history_sync_handlers = history_sync_callbacks.len(),
            offline_sync_preview_handlers = offline_sync_preview_callbacks.len(),
            offline_sync_completed_handlers = offline_sync_completed_callbacks.len(),
            device_list_update_handlers = device_list_update_callbacks.len(),
            business_status_update_handlers = business_status_update_callbacks.len(),
            stream_replaced_handlers = stream_replaced_callbacks.len(),
            temporary_ban_handlers = temporary_ban_callbacks.len(),
            connect_failure_handlers = connect_failure_callbacks.len(),
            stream_error_handlers = stream_error_callbacks.len(),
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
                let receipt_callbacks = Arc::clone(&receipt_callbacks);
                let undecryptable_message_callbacks = Arc::clone(&undecryptable_message_callbacks);
                let notification_callbacks = Arc::clone(&notification_callbacks);
                let chat_presence_callbacks = Arc::clone(&chat_presence_callbacks);
                let presence_callbacks = Arc::clone(&presence_callbacks);
                let picture_update_callbacks = Arc::clone(&picture_update_callbacks);
                let user_about_update_callbacks = Arc::clone(&user_about_update_callbacks);
                let joined_group_callbacks = Arc::clone(&joined_group_callbacks);
                let group_info_update_callbacks = Arc::clone(&group_info_update_callbacks);
                let contact_update_callbacks = Arc::clone(&contact_update_callbacks);
                let push_name_update_callbacks = Arc::clone(&push_name_update_callbacks);
                let self_push_name_updated_callbacks = Arc::clone(&self_push_name_updated_callbacks);
                let pin_update_callbacks = Arc::clone(&pin_update_callbacks);
                let mute_update_callbacks = Arc::clone(&mute_update_callbacks);
                let archive_update_callbacks = Arc::clone(&archive_update_callbacks);
                let mark_chat_as_read_update_callbacks = Arc::clone(&mark_chat_as_read_update_callbacks);
                let history_sync_callbacks = Arc::clone(&history_sync_callbacks);
                let offline_sync_preview_callbacks = Arc::clone(&offline_sync_preview_callbacks);
                let offline_sync_completed_callbacks = Arc::clone(&offline_sync_completed_callbacks);
                let device_list_update_callbacks = Arc::clone(&device_list_update_callbacks);
                let business_status_update_callbacks = Arc::clone(&business_status_update_callbacks);
                let stream_replaced_callbacks = Arc::clone(&stream_replaced_callbacks);
                let temporary_ban_callbacks = Arc::clone(&temporary_ban_callbacks);
                let connect_failure_callbacks = Arc::clone(&connect_failure_callbacks);
                let stream_error_callbacks = Arc::clone(&stream_error_callbacks);
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
                                    let payload = Py::new(py, EvPairingQrCode::new(code.clone(), timeout.as_secs()))?;
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
                            let payload = Python::attach(|py| Py::new(py, EvMessage::new(msg, info))).map_err(|e| e).unwrap();
                            for callback in message_callbacks.iter() {
                                let locals = locals.clone();
                                let py_future = Python::attach(|py| -> PyResult<_> {
                                    let client_obj = tryx_client.clone_ref(py);
                                    let awaitable = callback.bind(py).call1((client_obj, payload.clone_ref(py)))?;
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
                                            Python::attach(|py| err.print(py));
                                        } else {
                                            debug!( "message callback finished");
                                        }
                                    }
                                    Err(err) => {
                                        error!("failed to schedule message callback");
                                        Python::attach(|py| err.print(py));
                                    }
                                }
                            }
                        }
                        Event::Connected(_) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvConnected{})).map_err(|e| e).unwrap();
                            for callback in connected_callbacks.iter() {
                                debug!("calling connected event handler");
                                let _ = Python::attach(|py| -> PyResult<_> {
                                    let awaitable = callback.bind(py).call1((payload.clone_ref(py),))?;
                                    let fut = into_future(awaitable)?;
                                    Ok(fut)
                                });
                            }
                        }
                        Event::LoggedOut(logout) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvLoggedOut::new(logout))).map_err(|e| e).unwrap();
                            for callback in logout_callbacks.iter() {
                                debug!("calling logged out event handler");
                                let _ = Python::attach(|py| -> PyResult<_> {
                                    let awaitable = callback.bind(py).call1((payload.clone_ref(py),))?;
                                    let fut = into_future(awaitable)?;
                                    Ok(fut)
                                });
                            }
                            
                        }
                        Event::ArchiveUpdate(archived) => {

                            let payload = Python::attach(|py| pyo3::Py::new(py, EvArchiveUpdate::new(
                                archived.jid.into(),
                                archived.timestamp,
                                Arc::from(archived.action.clone()),
                                archived.from_full_sync,
                            ))).map_err(|e| e).unwrap();
                            for callback in archive_update_callbacks.iter() {
                                debug!("calling archive update event handler");
                                let _ = Python::attach(|py| -> PyResult<_> {
                                    let awaitable = callback.bind(py).call1((payload.clone_ref(py),))?;
                                    let fut = into_future(awaitable)?;
                                    Ok(fut)
                                });
                            }
                            // debug!("received archive update event for jid {}", archived.jid);

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