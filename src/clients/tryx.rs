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
use whatsapp_rust::{Client, TokioRuntime};
use whatsapp_rust::bot::Bot;
use whatsapp_rust::store::Backend;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;
use tokio::signal;
use tracing::{debug, error, info, warn};
use super::community::CommunityClient;
use super::contacts::ContactClient;
use super::chatstate::ChatstateClient;
use super::blocking::BlockingClient;
use super::groups::GroupsClient;
use super::newsletter::NewsletterClient;
use super::polls::PollsClient;
use super::presence::PresenceClient;
use super::privacy::PrivacyClient;
use super::profile::ProfileClient;
use super::status::StatusClient;
use super::tryx_client::TryxClient;
use crate::clients::chat_actions::ChatActionsClient;
use crate::log::init_logging;
use crate::backend::{SqliteBackend, BackendBase};
use crate::events::types::{
    EvArchiveUpdate, EvBusinessStatusUpdate, EvChatPresence, EvClientOutDated, EvConnectFailure, EvConnected, EvContactNumberChanged, EvContactSyncRequested, EvContactUpdate, EvContactUpdated, EvDeleteChatUpdate, EvDeleteMessageForMeUpdate, EvDeviceListUpdate, EvDisappearingModeChanged, EvDisconnected, EvGroupUpdate, EvHistorySync, EvLoggedOut, EvMarkChatAsReadUpdate, EvMessage, EvMuteUpdate, EvNewsletterLiveUpdate, EvNotification, EvOfflineSyncCompleted, EvOfflineSyncPreview, EvPairError, EvPairSuccess, EvPairingCode, EvPairingQrCode, EvPictureUpdate, EvPinUpdate, EvPresence, EvPushNameUpdate, EvQrScannedWithoutMultidevice, EvReceipt, EvSelfPushNameUpdated, EvStarUpdate, EvStreamError, EvStreamReplaced, EvTemporaryBan, EvUndecryptableMessage, EvUserAboutUpdate
};
use crate::exceptions::{EventDispatchError, FailedBuildClient, UnsupportedBackend};
use crate::events::dispatcher::Dispatcher;
use super::event_callbacks::EventCallbacks;

/// Creates a `Py<T>` namespace client that shares the `client_rx` watch channel.
macro_rules! new_namespace_client {
    ($py:expr, $rx:expr, $ty:ident) => {
        Py::new($py, $ty { client_rx: $rx.clone() })?
    };
}

type PyCallbackFuture = Pin<Box<dyn Future<Output = PyResult<Py<PyAny>>> + Send>>;


#[pyclass]
pub struct Tryx {
    backend: Arc<dyn Backend>,
    #[pyo3(get)]
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
            let tryx_client = Py::new(
                py,
                TryxClient {
                    client_rx: client_rx.clone(),
                    contact: new_namespace_client!(py, client_rx, ContactClient),
                    chat_actions: new_namespace_client!(py, client_rx, ChatActionsClient),
                    community: new_namespace_client!(py, client_rx, CommunityClient),
                    newsletter: new_namespace_client!(py, client_rx, NewsletterClient),
                    groups: new_namespace_client!(py, client_rx, GroupsClient),
                    status: new_namespace_client!(py, client_rx, StatusClient),
                    chatstate: new_namespace_client!(py, client_rx, ChatstateClient),
                    blocking: new_namespace_client!(py, client_rx, BlockingClient),
                    polls: new_namespace_client!(py, client_rx, PollsClient),
                    presence: new_namespace_client!(py, client_rx, PresenceClient),
                    privacy: new_namespace_client!(py, client_rx, PrivacyClient),
                    profile: new_namespace_client!(py, client_rx, ProfileClient),
                }
            )?;
            
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
    /// @client.on(EvMessage)
    /// async def on_message(client, event): ...
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
        info!("starting client in async mode via Tryx.run");
        let backend = self.backend.clone();
        let handlers = self.handlers.clone_ref(py);
        let tryx_client = self.tryx_client.clone_ref(py);
        let client_tx = self.client_tx.clone();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals.clone(), async move {
            Self::run_automation(backend, handlers, Some(locals), tryx_client, client_tx).await
        })
    
    }

    /// Starts the client and blocks until it exits.
    ///
    /// Python usage:
    /// client.run_blocking()
    fn run_blocking(&self, py: Python<'_>) -> PyResult<()> {
        init_logging();
        info!("starting client in blocking mode via Tryx.run_blocking");
        let backend = self.backend.clone();
        let handlers = self.handlers.clone_ref(py);
        let tryx_client = self.tryx_client.clone_ref(py);
        let client_tx = self.client_tx.clone();
        py.detach(move || {
            let rt = runtime::Runtime::new()
                .map_err(|e| {
                    error!(error = %e, "failed to create tokio runtime for blocking mode");
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                })?;

            rt.block_on(async {
                let mut task = tokio::spawn(Self::run_automation(backend, handlers, None, tryx_client, client_tx));
                let mut signal_tick = interval(Duration::from_millis(200));

                loop {
                    tokio::select! {
                        _ = signal::ctrl_c() => {
                            warn!("SIGINT received via tokio::signal, stopping task");
                            task.abort();
                            break;
                        }
                        _ = signal_tick.tick() => {
                            let signal_result = Python::attach(|py| {
                                match py.check_signals() {
                                    Ok(()) => Ok(()),
                                    Err(err) => {
                                        let is_keyboard_interrupt = err.is_instance_of::<pyo3::exceptions::PyKeyboardInterrupt>(py);
                                        Err((err, is_keyboard_interrupt))
                                    }
                                }
                            });
                            if let Err((err, is_keyboard_interrupt)) = signal_result {
                                if is_keyboard_interrupt {
                                    warn!("KeyboardInterrupt detected from Python, stopping task");
                                    task.abort();
                                    break;
                                }

                                error!(error = %err, "non-keyboard Python signal error while polling");
                                task.abort();
                                return Err(err);
                            }
                        }
                        result = &mut task => {
                            match result {
                                Ok(inner) => {
                                    info!("task finished in blocking mode");
                                    inner?;
                                }
                                Err(err) if err.is_cancelled() => {
                                    info!("task cancelled");
                                }
                                Err(err) => {
                                    error!(error = %err, "task join failed");
                                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(err.to_string()));
                                }
                            }
                            info!("blocking run finished");
                            return Ok(());
                        }
                    }
                }

                match task.await {
                    Ok(Ok(())) => info!("client finished after interrupt"),
                    Ok(Err(err)) => return Err(err),
                    Err(join_err) if join_err.is_cancelled() => info!("task cancelled successfully"),
                    Err(join_err) => {
                        error!(error = %join_err, "task join failed after interrupt");
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
            async fn call_event(callbacks: &[Py<PyAny>], tryx_client: &Py<TryxClient>, payload: Py<PyAny>, locals: Option<TaskLocals>) -> PyResult<()> {
            let py_futures = Python::attach(|py| -> PyResult<Vec<PyCallbackFuture>> {
                callbacks
                    .iter()
                    .map(|callback| {
                        debug!("scheduling event Python callback");
                            let awaitable = callback
                                .bind(py)
                                .call1((tryx_client.clone_ref(py), payload.clone_ref(py)))?;
                        let fut: PyCallbackFuture = match locals.as_ref() {
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
                    })
                    .collect()
            });

            match py_futures {
                Ok(futures) => {
                    for py_future in futures {
                        if let Err(err) = py_future.await {
                            error!(error = %err, "event callback failed");
                            Python::attach(|py| err.print(py));
                        } else {
                            debug!("event callback finished");
                        }
                }
                }
                Err(err) => {
                    error!(error = %err, "failed to schedule event callback");
                    Python::attach(|py| err.print(py));
            }
        }
        Ok(())
    }

        async fn emit_event(
        callbacks: &[Py<PyAny>],
        tryx_client: &Py<TryxClient>,
            payload: PyResult<Py<PyAny>>,
        locals: Option<TaskLocals>,
        event_name: &str,
    ) {
        match payload {
            Ok(py_payload) => {
                if let Err(err) = Self::call_event(callbacks, tryx_client, py_payload, locals).await {
                    error!(event = event_name, error = %err, "failed to call python event callbacks");
                    Python::attach(|py| err.print(py));
                }
            }
            Err(err) => {
                error!(event = event_name, error = %err, "failed to build python event payload");
                Python::attach(|py| err.print(py));
            }
        }
    }

    async fn emit_built_event<F>(
        tryx_client: &Py<TryxClient>,
        callbacks: &[Py<PyAny>],
        locals: Option<TaskLocals>,
        event_name: &str,
        build_payload: F,
    ) where
        F: FnOnce(Python<'_>) -> PyResult<Py<PyAny>>,
    {
        if callbacks.is_empty() {
            return;
        }
        let payload = Python::attach(build_payload);
        Self::emit_event(callbacks, tryx_client, payload, locals, event_name).await;
    }

    async fn run_automation(
        backend: Arc<dyn Backend>,
        handlers: Py<Dispatcher>,
        locals: Option<TaskLocals>,
        tryx_client: Py<TryxClient>,
        client_tx: watch::Sender<Option<Arc<Client>>>,
    ) -> PyResult<()> {
        let callbacks = Arc::new(Python::attach(|py| {
            let dispatcher = handlers.bind(py).borrow();
            EventCallbacks::from_dispatcher(py, &dispatcher)
        }));
        info!("building WhatsApp automation client");
        let mut automation = Bot::builder()
            .with_backend(backend)
            .with_transport_factory(TokioWebSocketTransportFactory::new())
            .with_http_client(UreqHttpClient::new())
            .on_event(move |event, _client| {
                let locals = locals.clone();
                let callbacks = Arc::clone(&callbacks);
                let tryx_client = Python::attach(|py| tryx_client.clone_ref(py));

                async move {
                    match event.as_ref() {
                        Event::Connected(_) => {
                            Self::emit_built_event(&tryx_client, &callbacks.connected, locals.clone(), "Connected", |py| {
                                Py::new(py, EvConnected {}).map(|event| event.into_any())
                            }).await;
                        }
                        Event::Disconnected(_) => {
                            Self::emit_built_event(&tryx_client, &callbacks.disconnected, locals.clone(), "Disconnected", |py| {
                                Py::new(py, EvDisconnected {}).map(|event| event.into_any())
                            }).await;
                        }
                        Event::LoggedOut(logout) => {
                            let logout = logout.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.logout, locals.clone(), "LoggedOut", |py| {
                                Py::new(py, EvLoggedOut::new(logout)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::PairSuccess(pair_success) => {
                            let pair_success = pair_success.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.pair_success, locals.clone(), "PairSuccess", |py| {
                                Py::new(py, EvPairSuccess::from(pair_success)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::PairError(pair_error) => {
                            let pair_error = pair_error.clone();
                            Self::emit_built_event(
                                &tryx_client,
                                &callbacks.pair_error,
                                locals.clone(),
                                "PairError",
                                |py| {
                                    Py::new(
                                        py,
                                        EvPairError::new(
                                            pair_error.id.into(),
                                            pair_error.lid.into(),
                                            pair_error.business_name,
                                            pair_error.platform,
                                            pair_error.error,
                                        ),
                                    )
                                    .map(|event| event.into_any())
                                },
                            )
                            .await;
                        }
                        Event::PairingQrCode { code, timeout } => {
                            let code = code.clone();
                            let timeout_secs = timeout.as_secs();
                            Self::emit_built_event(&tryx_client, &callbacks.pairing_qr, locals.clone(), "PairingQrCode", |py| {
                                Py::new(py, EvPairingQrCode::new(code, timeout_secs)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::PairingCode { code, timeout } => {
                            let code = code.clone();
                            let timeout_secs = timeout.as_secs();
                            Self::emit_built_event(&tryx_client, &callbacks.pairing_code, locals.clone(), "PairingCode", |py| {
                                Py::new(py, EvPairingCode::new(code, timeout_secs)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::QrScannedWithoutMultidevice(scanned) => {
                            let scanned = scanned.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.qr_scanned_without_multidevice, locals.clone(), "QrScannedWithoutMultidevice", |py| {
                                Py::new(py, EvQrScannedWithoutMultidevice::from(scanned)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ClientOutdated(_) => {
                            Self::emit_built_event(&tryx_client, &callbacks.client_outdated, locals.clone(), "ClientOutdated", |py| {
                                Py::new(py, EvClientOutDated {}).map(|event| event.into_any())
                            }).await;
                        }
                        Event::Message(msg, info) => {
                            let msg = (**msg).clone();
                            let info = (**info).clone();
                            Self::emit_built_event(&tryx_client, &callbacks.message, locals.clone(), "Message", |py| {
                                Py::new(py, EvMessage::new(msg, info)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::Receipt(receipt) => {
                            let receipt = receipt.clone();
                            Self::emit_built_event(
                                &tryx_client,
                                &callbacks.receipt,
                                locals.clone(),
                                "Receipt",
                                |_py| {
                                    Ok(EvReceipt::new(
                                        receipt.source,
                                        receipt.message_ids,
                                        receipt.timestamp,
                                        receipt.r#type,
                                    )
                                    .into_any())
                                },
                            )
                            .await;
                        }
                        Event::UndecryptableMessage(undecryptable_message) => {
                            let info = (*undecryptable_message.info).clone();
                            let is_unavailable = undecryptable_message.is_unavailable;
                            let unavailable_type = undecryptable_message.unavailable_type.clone();
                            let decrypt_fail_mode = undecryptable_message.decrypt_fail_mode.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.undecryptable_message, locals.clone(), "UndecryptableMessage", |py| {
                                Py::new(py, EvUndecryptableMessage::new(info, is_unavailable, unavailable_type, decrypt_fail_mode)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::Notification(notification) => {
                            let notification = notification.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.notification, locals.clone(), "Notification", |py| {
                                Py::new(py, EvNotification::new(notification.to_owned_node())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ChatPresence(chat_presence) => {
                            let chat_presence = chat_presence.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.chat_presence, locals.clone(), "ChatPresence", |py| {
                                Py::new(py, EvChatPresence::from(chat_presence)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::Presence(presence) => {
                            let presence = presence.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.presence, locals.clone(), "Presence", |py| {
                                Py::new(py, EvPresence::from(presence)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::PictureUpdate(picture_update) => {
                            let picture_update = picture_update.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.picture_update, locals.clone(), "PictureUpdate", |py| {
                                Py::new(py, EvPictureUpdate::new(picture_update)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::UserAboutUpdate(user_about) => {
                            let user_about = user_about.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.user_about_update, locals.clone(), "UserAboutUpdate", |py| {
                                Py::new(py, EvUserAboutUpdate::new(user_about)).map(|event| event.into_any())
                            }).await;
                        }
                        // JoinedGroup is not a current Event variant; reserved for future use.
                        Event::GroupUpdate(group_info) => {
                            let group_info = group_info.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.group_info_update, locals.clone(), "GroupUpdate", |py| {
                                Py::new(py, EvGroupUpdate::new(group_info)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ContactUpdate(contact_update) => {
                            let contact_update = contact_update.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.contact_update, locals.clone(), "ContactUpdate", |py| {
                                Py::new(py, EvContactUpdate::new(contact_update)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::PushNameUpdate(pushname) => {
                            let pushname = pushname.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.push_name_update, locals.clone(), "PushNameUpdate", |py| {
                                Py::new(py, EvPushNameUpdate::from(pushname)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::SelfPushNameUpdated(self_push_name_update) => {
                            let self_push_name_update = self_push_name_update.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.self_push_name_updated, locals.clone(), "SelfPushNameUpdated", |py| {
                                Py::new(py, EvSelfPushNameUpdated::from(self_push_name_update)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::PinUpdate(pin_update) => {
                            let pin_update = pin_update.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.pin_update, locals.clone(), "PinUpdate", |py| {
                                Py::new(py, EvPinUpdate::new(pin_update)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::MuteUpdate(mute_update) => {
                            let mute_update = mute_update.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.mute_update, locals.clone(), "MuteUpdate", |py| {
                                Py::new(py, EvMuteUpdate::from(mute_update)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ArchiveUpdate(archived) => {
                            let archived = archived.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.archive_update, locals.clone(), "ArchiveUpdate", |py| {
                                Py::new(py, EvArchiveUpdate::from(archived)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::MarkChatAsReadUpdate(mark_chat_as_read_update) => {
                            let mark_chat_as_read_update = mark_chat_as_read_update.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.mark_chat_as_read_update, locals.clone(), "MarkChatAsReadUpdate", |py| {
                                Py::new(py, EvMarkChatAsReadUpdate::from(mark_chat_as_read_update)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::HistorySync(history_sync) => {
                            let history_sync = history_sync.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.history_sync, locals.clone(), "HistorySync", |py| {
                                if let Some(decoded) = history_sync.get() {
                                    Py::new(py, EvHistorySync::from(decoded.clone())).map(|event| event.into_any())
                                } else {
                                    Err(pyo3::exceptions::PyRuntimeError::new_err("Failed to decode HistorySync"))
                                }
                            }).await;
                        }
                        Event::OfflineSyncPreview(offline_sync_preview) => {
                            let offline_sync_preview = offline_sync_preview.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.offline_sync_preview, locals.clone(), "OfflineSyncPreview", |py| {
                                Py::new(py, EvOfflineSyncPreview::from(offline_sync_preview)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::OfflineSyncCompleted(offline_sync_complete) => {
                            let offline_sync_complete = offline_sync_complete.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.offline_sync_completed, locals.clone(), "OfflineSyncCompleted", |py| {
                                Py::new(py, EvOfflineSyncCompleted::from(offline_sync_complete)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::DeviceListUpdate(device_list_update) => {
                            let device_list_update = device_list_update.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.device_list_update, locals.clone(), "DeviceListUpdate", |py| {
                                Py::new(py, EvDeviceListUpdate::from(device_list_update)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::BusinessStatusUpdate(business_status_update) => {
                            let business_status_update = business_status_update.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.business_status_update, locals.clone(), "BusinessStatusUpdate", |py| {
                                Py::new(py, EvBusinessStatusUpdate::from(business_status_update)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::StreamReplaced(_) => {
                            Self::emit_built_event(&tryx_client, &callbacks.stream_replaced, locals.clone(), "StreamReplaced", |py| {
                                Py::new(py, EvStreamReplaced {}).map(|event| event.into_any())
                            }).await;
                        }
                        Event::TemporaryBan(temporary_ban) => {
                            let temporary_ban = temporary_ban.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.temporary_ban, locals.clone(), "TemporaryBan", |py| {
                                Py::new(py, EvTemporaryBan::from(temporary_ban)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ConnectFailure(connect_failure) => {
                            let connect_failure = connect_failure.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.connect_failure, locals.clone(), "ConnectFailure", |py| {
                                Py::new(py, EvConnectFailure::new(connect_failure.reason, connect_failure.message, connect_failure.raw)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::StreamError(stream_error) => {
                            let stream_error = stream_error.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.stream_error, locals.clone(), "StreamError", |py| {
                                Py::new(py, EvStreamError::new(stream_error.code, stream_error.raw)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ContactNumberChanged(contact_number_changed) => {
                            let contact_number_changed = contact_number_changed.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.contact_number_changed, locals.clone(), "ContactNumberChanged", |py| {
                                Py::new(py, EvContactNumberChanged::from(contact_number_changed)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ContactSyncRequested(contact_sync_requested) => {
                            let contact_sync_requested = contact_sync_requested.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.contact_sync_requested, locals.clone(), "ContactSyncRequested", |py| {
                                Py::new(py, EvContactSyncRequested::from(contact_sync_requested)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ContactUpdated(contact_updated) => {
                            let contact_updated = contact_updated.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.contact_updated, locals.clone(), "ContactUpdated", |py| {
                                Py::new(py, EvContactUpdated::from(contact_updated)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::StarUpdate(star_update) => {
                            let star_update = star_update.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.star_update, locals.clone(), "StarUpdate", |py| {
                                Py::new(py, EvStarUpdate::from(star_update)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::DisappearingModeChanged(disappearing_mode_changed) => {
                            let disappearing_mode_changed = disappearing_mode_changed.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.disappearing_mode_changed, locals.clone(), "DisappearingModeChanged", |py| {
                                Py::new(py, EvDisappearingModeChanged::from(disappearing_mode_changed)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::NewsletterLiveUpdate(newsletter_live_update) => {
                            let newsletter_live_update = newsletter_live_update.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.newsletter_live_update, locals.clone(), "NewsletterLiveUpdate", |py| {
                                Py::new(py, EvNewsletterLiveUpdate::from(newsletter_live_update)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::DeleteChatUpdate(delete_chat_update) => {
                            let delete_chat_update = delete_chat_update.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.delete_chat_update, locals.clone(), "DeleteChatUpdate", |py| {
                                Py::new(py, EvDeleteChatUpdate::from(delete_chat_update)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::DeleteMessageForMeUpdate(delete_message_for_me_update) => {
                            let delete_message_for_me_update = delete_message_for_me_update.clone();
                            Self::emit_built_event(&tryx_client, &callbacks.delete_message_for_me_update, locals.clone(), "DeleteMessageForMeUpdate", |py| {
                                Py::new(py, EvDeleteMessageForMeUpdate::from(delete_message_for_me_update)).map(|event| event.into_any())
                            }).await;
                        }
                        _ => {
                            debug!("received unsupported event type: {:?}", event);
                        }
                }
            }
            })
            .with_runtime(TokioRuntime)
            .build()
            .await
            .map_err(|e| {
                error!(error = %e, "failed to build client");
                PyErr::new::<FailedBuildClient, _>(e.to_string())
            })?;

        let client = automation.client();
        client_tx
            .send(Some(client))
            .map_err(|e| PyErr::new::<EventDispatchError, _>(e.to_string()))?;

        info!("client built successfully, starting run loop");
        automation.run()
            .await
            .map_err(|e| {
                error!(error = %e, "failed to start run stream");
                PyErr::new::<EventDispatchError, _>(e.to_string())
            })?
            .await
            .map_err(|e| {
                error!(error = %e, "run stream failed");
                PyErr::new::<EventDispatchError, _>(e.to_string())
            })?;

        info!("run loop finished");

        Ok(())
    }
}