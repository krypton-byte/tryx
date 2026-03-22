use std::sync::{Arc};
use std::future::Future;
use std::pin::Pin;
use pyo3::{Bound, PyAny, PyTypeInfo, pyclass, pymethods};
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
use super::tryx_client::TryxClient;
use crate::log::init_logging;
use crate::backend::{SqliteBackend, BackendBase};
use crate::events::types::{
    EvArchiveUpdate, EvBusinessStatusUpdate, EvChatPresence, EvClientOutDated, EvConnectFailure, EvConnected, EvContactNumberChanged, EvContactSyncRequested, EvContactUpdate, EvContactUpdated, EvDeleteChatUpdate, EvDeviceListUpdate, EvDisappearingModeChanged, EvDisconnected, EvGroupUpdate, EvHistorySync, EvJoinedGroup, EvLoggedOut, EvMarkChatAsReadUpdate, EvMessage, EvMuteUpdate, EvNewsletterLiveUpdate, EvNotification, EvOfflineSyncCompleted, EvOfflineSyncPreview, EvPairError, EvPairSuccess, EvPairingCode, EvPairingQrCode, EvPictureUpdate, EvPinUpdate, EvPresence, EvPushNameUpdate, EvQrScannedWithoutMultidevice, EvReceipt, EvSelfPushNameUpdated, EvStarUpdate, EvStreamError, EvStreamReplaced, EvTemporaryBan, EvUndecryptableMessage, EvUserAboutUpdate
};
use crate::exceptions::{EventDispatchError, FailedBuildBot, UnsupportedBackend};
use crate::events::dispatcher::Dispatcher;
use super::event_callbacks::EventCallbacks;


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
    /// @client.on(EvMessage)
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
    async fn call_event<T: PyTypeInfo>(callbacks: &[Py<PyAny>], payload: Py<T>, locals: Option<TaskLocals>) -> PyResult<()> {
        for callback in callbacks.iter() {
            debug!("calling event Python callback");
            let py_future = Python::attach(|py| -> PyResult<_> {
                let awaitable = callback.bind(py).call1((payload.clone_ref(py),))?;
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
                        error!(error = %err, "event callback failed");
                        Python::attach(|py| err.print(py));
                    } else {
                        debug!("event callback finished");
                    }
                }
                Err(err) => {
                    error!(error = %err, "failed to schedule event callback");
                    Python::attach(|py| err.print(py));
                }
            }
        }
        Ok(())
    }

    async fn emit_event<T: PyTypeInfo>(
        callbacks: &[Py<PyAny>],
        payload: PyResult<Py<T>>,
        locals: Option<TaskLocals>,
        event_name: &str,
    ) {
        match payload {
            Ok(py_payload) => {
                if let Err(err) = Self::call_event(callbacks, py_payload, locals).await {
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

    async fn run_bot(
        backend: Arc<dyn Backend>,
        handlers: Py<Dispatcher>,
        locals: Option<TaskLocals>,
        _tryx_client: Py<TryxClient>,
        client_tx: watch::Sender<Option<Arc<Client>>>,
    ) -> PyResult<()> {
        let callbacks = Arc::new(Python::attach(|py| {
            let dispatcher = handlers.bind(py).borrow();
            EventCallbacks::from_dispatcher(py, &dispatcher)
        }));
        info!("building WhatsApp bot");
        let mut bot = Bot::builder()
            .with_backend(backend)
            .with_transport_factory(TokioWebSocketTransportFactory::new())
            .with_http_client(UreqHttpClient::new())
            .on_event(move |event, _client| {
                let locals = locals.clone();
                let callbacks = Arc::clone(&callbacks);

                async move {
                    match event {
                        Event::Connected(_) => {
                            Self::emit_event(&callbacks.connected, Python::attach(|py| Py::new(py, EvConnected {})), locals.clone(), "Connected").await;
                        }
                        Event::Disconnected(_) => {
                            Self::emit_event(&callbacks.disconnected, Python::attach(|py| Py::new(py, EvDisconnected {})), locals.clone(), "Disconnected").await;
                        }
                        Event::LoggedOut(logout) => {
                            Self::emit_event(&callbacks.logout, Python::attach(|py| Py::new(py, EvLoggedOut::new(logout))), locals.clone(), "LoggedOut").await;
                        }
                        Event::PairSuccess(pair_success) => {
                            Self::emit_event(&callbacks.pair_success, Python::attach(|py| Py::new(py, EvPairSuccess::from(pair_success))), locals.clone(), "PairSuccess").await;
                        }
                        Event::PairError(pair_error) => {
                            Self::emit_event(
                                &callbacks.pair_error,
                                Python::attach(|py| {
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
                                }),
                                locals.clone(),
                                "PairError",
                            )
                            .await;
                        }
                        Event::PairingQrCode { code, timeout } => {
                            Self::emit_event(&callbacks.pairing_qr, Python::attach(|py| Py::new(py, EvPairingQrCode::new(code, timeout.as_secs()))), locals.clone(), "PairingQrCode").await;
                        }
                        Event::PairingCode { code, timeout } => {
                            Self::emit_event(&callbacks.pairing_code, Python::attach(|py| Py::new(py, EvPairingCode::new(code, timeout.as_secs()))), locals.clone(), "PairingCode").await;
                        }
                        Event::QrScannedWithoutMultidevice(scanned) => {
                            Self::emit_event(&callbacks.qr_scanned_without_multidevice, Python::attach(|py| Py::new(py, EvQrScannedWithoutMultidevice::from(scanned))), locals.clone(), "QrScannedWithoutMultidevice").await;
                        }
                        Event::ClientOutdated(_) => {
                            Self::emit_event(&callbacks.client_outdated, Python::attach(|py| Py::new(py, EvClientOutDated {})), locals.clone(), "ClientOutdated").await;
                        }
                        Event::Message(msg, info) => {
                            Self::emit_event(&callbacks.message, Python::attach(|py| Py::new(py, EvMessage::new(*msg, info))), locals.clone(), "Message").await;
                        }
                        Event::Receipt(receipt) => {
                            Self::emit_event(
                                &callbacks.receipt,
                                Ok(EvReceipt::new(
                                    receipt.source,
                                    receipt.message_ids,
                                    receipt.timestamp,
                                    receipt.r#type,
                                    receipt.message_sender,
                                )),
                                locals.clone(),
                                "Receipt",
                            )
                            .await;
                        }
                        Event::UndecryptableMessage(undecryptable_message) => {
                            Self::emit_event(&callbacks.undecryptable_message, Python::attach(|py| Py::new(py, EvUndecryptableMessage::new(undecryptable_message.info.clone(), undecryptable_message.is_unavailable, undecryptable_message.unavailable_type, undecryptable_message.decrypt_fail_mode))), locals.clone(), "UndecryptableMessage").await;
                        }
                        Event::Notification(notification) => {
                            Self::emit_event(&callbacks.notification, Python::attach(|py| Py::new(py, EvNotification::new(notification))), locals.clone(), "Notification").await;
                        }
                        Event::ChatPresence(chat_presence) => {
                            Self::emit_event(&callbacks.chat_presence, Python::attach(|py| Py::new(py, EvChatPresence::from(chat_presence))), locals.clone(), "ChatPresence").await;
                        }
                        Event::Presence(presence) => {
                            Self::emit_event(&callbacks.presence, Python::attach(|py| Py::new(py, EvPresence::from(presence))), locals.clone(), "Presence").await;
                        }
                        Event::PictureUpdate(picture_update) => {
                            Self::emit_event(&callbacks.picture_update, Python::attach(|py| Py::new(py, EvPictureUpdate::new(picture_update))), locals.clone(), "PictureUpdate").await;
                        }
                        Event::UserAboutUpdate(user_about) => {
                            Self::emit_event(&callbacks.user_about_update, Python::attach(|py| Py::new(py, EvUserAboutUpdate::new(user_about))), locals.clone(), "UserAboutUpdate").await;
                        }
                        Event::JoinedGroup(joined_group) => {
                            Self::emit_event(&callbacks.joined_group, Python::attach(|py| Py::new(py, EvJoinedGroup::new(joined_group))), locals.clone(), "JoinedGroup").await;
                        }
                        Event::GroupUpdate(group_info) => {
                            Self::emit_event(&callbacks.group_info_update, Python::attach(|py| Py::new(py, EvGroupUpdate::new(group_info))), locals.clone(), "GroupUpdate").await;
                        }
                        Event::ContactUpdate(contact_update) => {
                            Self::emit_event(&callbacks.contact_update, Python::attach(|py| Py::new(py, EvContactUpdate::new(contact_update))), locals.clone(), "ContactUpdate").await;
                        }
                        Event::PushNameUpdate(pushname) => {
                            Self::emit_event(&callbacks.push_name_update, Python::attach(|py| Py::new(py, EvPushNameUpdate::from(pushname))), locals.clone(), "PushNameUpdate").await;
                        }
                        Event::SelfPushNameUpdated(self_push_name_update) => {
                            Self::emit_event(&callbacks.self_push_name_updated, Python::attach(|py| Py::new(py, EvSelfPushNameUpdated::from(self_push_name_update))), locals.clone(), "SelfPushNameUpdated").await;
                        }
                        Event::PinUpdate(pin_update) => {
                            Self::emit_event(&callbacks.pin_update, Python::attach(|py| Py::new(py, EvPinUpdate::new(pin_update))), locals.clone(), "PinUpdate").await;
                        }
                        Event::MuteUpdate(mute_update) => {
                            Self::emit_event(&callbacks.mute_update, Python::attach(|py| Py::new(py, EvMuteUpdate::from(mute_update))), locals.clone(), "MuteUpdate").await;
                        }
                        Event::ArchiveUpdate(archived) => {
                            Self::emit_event(&callbacks.archive_update, Python::attach(|py| Py::new(py, EvArchiveUpdate::from(archived))), locals.clone(), "ArchiveUpdate").await;
                        }
                        Event::MarkChatAsReadUpdate(mark_chat_as_read_update) => {
                            Self::emit_event(&callbacks.mark_chat_as_read_update, Python::attach(|py| Py::new(py, EvMarkChatAsReadUpdate::from(mark_chat_as_read_update))), locals.clone(), "MarkChatAsReadUpdate").await;
                        }
                        Event::HistorySync(history_sync) => {
                            Self::emit_event(&callbacks.history_sync, Python::attach(|py| Py::new(py, EvHistorySync::from(history_sync))), locals.clone(), "HistorySync").await;
                        }
                        Event::OfflineSyncPreview(offline_sync_preview) => {
                            Self::emit_event(&callbacks.offline_sync_preview, Python::attach(|py| Py::new(py, EvOfflineSyncPreview::from(offline_sync_preview))), locals.clone(), "OfflineSyncPreview").await;
                        }
                        Event::OfflineSyncCompleted(offline_sync_complete) => {
                            Self::emit_event(&callbacks.offline_sync_completed, Python::attach(|py| Py::new(py, EvOfflineSyncCompleted::from(offline_sync_complete))), locals.clone(), "OfflineSyncCompleted").await;
                        }
                        Event::DeviceListUpdate(device_list_update) => {
                            Self::emit_event(&callbacks.device_list_update, Python::attach(|py| Py::new(py, EvDeviceListUpdate::from(device_list_update))), locals.clone(), "DeviceListUpdate").await;
                        }
                        Event::BusinessStatusUpdate(business_status_update) => {
                            Self::emit_event(&callbacks.business_status_update, Python::attach(|py| Py::new(py, EvBusinessStatusUpdate::from(business_status_update))), locals.clone(), "BusinessStatusUpdate").await;
                        }
                        Event::StreamReplaced(_) => {
                            Self::emit_event(&callbacks.stream_replaced, Python::attach(|py| Py::new(py, EvStreamReplaced {})), locals.clone(), "StreamReplaced").await;
                        }
                        Event::TemporaryBan(temporary_ban) => {
                            Self::emit_event(&callbacks.temporary_ban, Python::attach(|py| Py::new(py, EvTemporaryBan::from(temporary_ban))), locals.clone(), "TemporaryBan").await;
                        }
                        Event::ConnectFailure(connect_failure) => {
                            Self::emit_event(&callbacks.connect_failure, Python::attach(|py| Py::new(py, EvConnectFailure::new(connect_failure.reason, connect_failure.message, connect_failure.raw))), locals.clone(), "ConnectFailure").await;
                        }
                        Event::StreamError(stream_error) => {
                            Self::emit_event(&callbacks.stream_error, Python::attach(|py| Py::new(py, EvStreamError::new(stream_error.code, stream_error.raw))), locals.clone(), "StreamError").await;
                        }
                        Event::ContactNumberChanged(contact_number_changed) => {
                            Self::emit_event(&callbacks.contact_number_changed, Python::attach(|py| Py::new(py, EvContactNumberChanged::from(contact_number_changed))), locals.clone(), "ContactNumberChanged").await;
                        }
                        Event::ContactSyncRequested(contact_sync_requested) => {
                            Self::emit_event(&callbacks.contact_sync_requested, Python::attach(|py| Py::new(py, EvContactSyncRequested::from(contact_sync_requested))), locals.clone(), "ContactSyncRequested").await;
                        }
                        Event::ContactUpdated(contact_updated) => {
                            Self::emit_event(&callbacks.contact_updated, Python::attach(|py| Py::new(py, EvContactUpdated::from(contact_updated))), locals.clone(), "ContactUpdated").await;
                        }
                        Event::StarUpdate(star_update) => {
                            Self::emit_event(&callbacks.star_update, Python::attach(|py| Py::new(py, EvStarUpdate::from(star_update))), locals.clone(), "StarUpdate").await;
                        }
                        Event::DisappearingModeChanged(disappearing_mode_changed) => {
                            Self::emit_event(&callbacks.disappearing_mode_changed, Python::attach(|py| Py::new(py, EvDisappearingModeChanged::from(disappearing_mode_changed))), locals.clone(), "DisappearingModeChanged").await;
                        }
                        Event::NewsletterLiveUpdate(newsletter_live_update) => {
                            // No current Python event for this, skipping
                            Self::emit_event(&callbacks.newsletter_live_update, Python::attach(|py| Py::new(py, EvNewsletterLiveUpdate::from(newsletter_live_update))), locals.clone(), "NewsletterLiveUpdate").await;
                        }
                        Event::DeleteChatUpdate(delete_chat_update) => {
                            Self::emit_event(&callbacks.delete_chat_update, Python::attach(|py| Py::new(py, EvDeleteChatUpdate::from(delete_chat_update))), locals.clone(), "DeleteChatUpdate").await;
                        }
                        Event::DeleteMessageForMeUpdate(delete_message_for_me_update) => {

                        }
                }
            }
            })
            .with_runtime(TokioRuntime)
            .build()
            .await
            .map_err(|e| {
                error!(error = %e, "failed to build bot");
                PyErr::new::<FailedBuildBot, _>(e.to_string())
            })?;

        let client = bot.client();
        client_tx
            .send(Some(client))
            .map_err(|e| PyErr::new::<EventDispatchError, _>(e.to_string()))?;

        info!("bot built successfully, starting run loop");
        bot.run()
            .await
            .map_err(|e| {
                error!(error = %e, "failed to start bot run stream");
                PyErr::new::<EventDispatchError, _>(e.to_string())
            })?
            .await
            .map_err(|e| {
                error!(error = %e, "bot run stream failed");
                PyErr::new::<EventDispatchError, _>(e.to_string())
            })?;

        info!("bot run loop finished");

        Ok(())
    }
}