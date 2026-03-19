use std::sync::{Arc};
use std::future::Future;
use std::pin::Pin;
use pyo3::{Bound, PyAny, PyClass, PyTypeInfo, pyclass, pymethods};
use pyo3::prelude::*;
use pyo3_async_runtimes::{TaskLocals, into_future_with_locals};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals, into_future};
use tokio::runtime;
use tokio::sync::watch;
use tokio::time::{Duration, interval};
use wacore::types::events::Event;
use waproto::whatsapp::disappearing_mode;
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
use crate::events::types::{
    EvArchiveUpdate, EvBusinessStatusUpdate, EvChatPresence, EvClientOutDated, EvConnectFailure, EvConnected, EvContactNumberChanged, EvContactSyncRequested, EvContactUpdate, EvContactUpdated, EvDeviceListUpdate, EvDisappearingModeChanged, EvDisconnected, EvGroupInfoUpdate, EvGroupUpdate, EvHistorySync, EvJoinedGroup, EvLoggedOut, EvMarkChatAsReadUpdate, EvMessage, EvMuteUpdate, EvNotification, EvOfflineSyncCompleted, EvOfflineSyncPreview, EvPairError, EvPairSuccess, EvPairingCode, EvPairingQrCode, EvPictureUpdate, EvPinUpdate, EvPresence, EvPushNameUpdate, EvQrScannedWithoutMultidevice, EvReceipt, EvSelfPushNameUpdated, EvStarUpdate, EvStreamError, EvStreamReplaced, EvTemporaryBan, EvUndecryptableMessage, EvUserAboutUpdate
};
use crate::exceptions::UnsupportedBackend;
use crate::events::dispatcher::Dispatcher;


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
    async fn call_event<T: PyTypeInfo>(callbacks: Arc<Vec<Py<PyAny>>>, payload: Py<T>, locals: Option<TaskLocals>) -> PyResult<()> {
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
    async fn run_bot(
        backend: Arc<dyn Backend>,
        handlers: Py<Dispatcher>,
        locals: Option<TaskLocals>,
        tryx_client: Py<TryxClient>,
        client_tx: watch::Sender<Option<Arc<Client>>>,
    ) -> PyResult<()> {
        let (
            connected_callbacks,
            disconnected_callbacks,
            logout_callbacks,
            pair_success_callbacks,
            pair_error_callbacks,
            pairing_qr_callbacks,
            pairing_code_callbacks,
            qr_scanned_without_multidevice_callbacks,
            client_outdated_callbacks,
            message_callbacks,
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
            disappearing_mode_changed_callbacks,
            contact_number_changed_callbacks,
            contact_sync_requested_callbacks,
            contact_updated_callbacks,
            star_update_callbacks,
        ) = Python::attach(|py| {
            let dispatcher = handlers.bind(py).borrow();
            (
                dispatcher.connected_handlers(py),
                dispatcher.disconnected_handlers(py),
                dispatcher.logout_handlers(py),
                dispatcher.pair_success_handlers(py),
                dispatcher.pair_error_handlers(py),
                dispatcher.pairing_qr_handlers(py),
                dispatcher.pairing_code_handlers(py),
                dispatcher.qr_scanned_without_multidevice_handlers(py),
                dispatcher.client_outdated_handlers(py),
                dispatcher.message_handlers(py),
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
                dispatcher.disappearing_mode_changed_handlers(py),
                dispatcher.contact_number_changed_handlers(py),
                dispatcher.contact_sync_requested_handlers(py),
                dispatcher.contact_updated_handlers(py),
                dispatcher.star_update_handlers(py),
            )
        });

        let connected_callbacks = Arc::new(connected_callbacks);
        let disconnected_callbacks = Arc::new(disconnected_callbacks);
        let logout_callbacks = Arc::new(logout_callbacks);
        let pair_success_callbacks = Arc::new(pair_success_callbacks);
        let pair_error_callbacks = Arc::new(pair_error_callbacks);
        let pairing_qr_callbacks = Arc::new(pairing_qr_callbacks);
        let pairing_code_callbacks = Arc::new(pairing_code_callbacks);
        let qr_scanned_without_multidevice_callbacks = Arc::new(qr_scanned_without_multidevice_callbacks);
        let client_outdated_callbacks = Arc::new(client_outdated_callbacks);
        let message_callbacks = Arc::new(message_callbacks);
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
        let disappearing_mode_changed_callbacks = Arc::new(disappearing_mode_changed_callbacks);
        let contact_number_changed_callbacks = Arc::new(contact_number_changed_callbacks);
        let contact_updated_callbacks = Arc::new(contact_updated_callbacks);
        let star_update_callbacks = Arc::new(star_update_callbacks);
        let contact_sync_requested_callbacks = Arc::new(contact_sync_requested_callbacks);
        info!("building WhatsApp bot");
        let mut bot = Bot::builder()
            .with_backend(backend)
            .with_transport_factory(TokioWebSocketTransportFactory::new())
            .with_http_client(UreqHttpClient::new())
            .on_event(move |event, _client| {
                let locals = locals.clone();
                let connected_callbacks = Arc::clone(&connected_callbacks);
                let disconnected_callbacks = Arc::clone(&disconnected_callbacks);
                let logout_callbacks = Arc::clone(&logout_callbacks);
                let pair_success_callbacks = Arc::clone(&pair_success_callbacks);
                let pair_error_callbacks = Arc::clone(&pair_error_callbacks);
                let pairing_qr_callbacks = Arc::clone(&pairing_qr_callbacks);
                let pairing_code_callbacks = Arc::clone(&pairing_code_callbacks);
                let qr_scanned_without_multidevice_callbacks = Arc::clone(&qr_scanned_without_multidevice_callbacks);
                let client_outdated_callbacks = Arc::clone(&client_outdated_callbacks);
                let message_callbacks = Arc::clone(&message_callbacks);
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
                let disappearing_mode_changed_callbacks = Arc::clone(&disappearing_mode_changed_callbacks);
                let contact_sync_requested_callbacks = Arc::clone(&contact_sync_requested_callbacks);
                let contact_updated_callbacks = Arc::clone(&contact_updated_callbacks);
                let contact_number_changed_callbacks = Arc::clone(&contact_number_changed_callbacks);
                let star_update_callbacks = Arc::clone(&star_update_callbacks);
                let _tryx_client = Python::attach(|py| tryx_client.clone_ref(py));

                async move {
                    match event {
                        Event::Connected(_) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvConnected {})).map_err(|e| e).unwrap();
                            Self::call_event(connected_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::Disconnected(_) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvDisconnected {})).map_err(|e| e).unwrap();
                            Self::call_event(disconnected_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::LoggedOut(logout) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvLoggedOut::new(logout))).map_err(|e| e).unwrap();
                            Self::call_event(logout_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::PairSuccess(pair_success) => {
                            let payload = Python::attach(|py| {
                                pyo3::Py::new(
                                    py,
                                    EvPairSuccess::new(
                                        pair_success.id.into(),
                                        pair_success.lid.into(),
                                        pair_success.business_name,
                                        pair_success.platform,
                                    ),
                                )
                            })
                            .map_err(|e| e)
                            .unwrap();
                            Self::call_event(pair_success_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::PairError(pair_error) => {
                            let payload = Python::attach(|py| {
                                pyo3::Py::new(
                                    py,
                                    EvPairError::new(
                                        pair_error.id.into(),
                                        pair_error.lid.into(),
                                        pair_error.business_name,
                                        pair_error.platform,
                                        pair_error.error,
                                    ),
                                )
                            })
                            .map_err(|e| e)
                            .unwrap();
                            Self::call_event(pair_error_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::PairingQrCode { code, timeout } => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvPairingQrCode::new(code, timeout.as_secs())))
                                .map_err(|e| e)
                                .unwrap();
                            Self::call_event(pairing_qr_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::PairingCode { code, timeout } => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvPairingCode::new(code, timeout.as_secs())))
                                .map_err(|e| e)
                                .unwrap();
                            Self::call_event(pairing_code_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::QrScannedWithoutMultidevice(scanned) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvQrScannedWithoutMultidevice::from(scanned)))
                                .map_err(|e| e)
                                .unwrap();
                            Self::call_event(qr_scanned_without_multidevice_callbacks, payload, locals.clone())
                                .await
                                .unwrap();
                        }
                        Event::ClientOutdated(_) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvClientOutDated {})).map_err(|e| e).unwrap();
                            Self::call_event(client_outdated_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::Message(msg, info) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvMessage::new(msg, info))).map_err(|e| e).unwrap();
                            Self::call_event(message_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::Receipt(receipt) => {
                            let payload = EvReceipt::new(
                                Arc::new(receipt.source),
                                receipt.message_ids,
                                receipt.timestamp,
                                receipt.r#type,
                                receipt.message_sender,
                            );
                            Self::call_event(receipt_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::UndecryptableMessage(undecryptable_message) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvUndecryptableMessage::new(Arc::new(undecryptable_message.info.clone()), undecryptable_message.is_unavailable, undecryptable_message.unavailable_type, undecryptable_message.decrypt_fail_mode)))
                                .map_err(|e| e)
                                .unwrap();
                            Self::call_event(undecryptable_message_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::Notification(notification) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvNotification::new(notification))).map_err(|e| e).unwrap();
                            Self::call_event(notification_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::ChatPresence(chat_presence) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvChatPresence::from(chat_presence))).map_err(|e| e).unwrap();
                            Self::call_event(chat_presence_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::Presence(presence) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvPresence::from(presence))).map_err(|e| e).unwrap();
                            Self::call_event(presence_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::PictureUpdate(picture_update) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvPictureUpdate::new(picture_update))).map_err(|e| e).unwrap();
                            Self::call_event(picture_update_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::UserAboutUpdate(user_about) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvUserAboutUpdate::new(user_about))).map_err(|e| e).unwrap();
                            Self::call_event(user_about_update_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::JoinedGroup(joined_group) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvJoinedGroup::new(joined_group))).map_err(|e| e).unwrap();
                            Self::call_event(joined_group_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::GroupUpdate(group_info) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvGroupUpdate::new(group_info))).map_err(|e| e).unwrap();
                            Self::call_event(group_info_update_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::ContactUpdate(contact_update) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvContactUpdate::new(contact_update))).map_err(|e| e).unwrap();
                            Self::call_event(contact_update_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::PushNameUpdate(pushname) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvPushNameUpdate::new(pushname.jid.into(), (*pushname.message.as_ref()).clone().into(), pushname.old_push_name.into(), pushname.new_push_name))).map_err(|e| e).unwrap();
                            Self::call_event(push_name_update_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::SelfPushNameUpdated(self_push_name_update) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvSelfPushNameUpdated::from(self_push_name_update))).map_err(|e| e).unwrap();
                            Self::call_event(self_push_name_updated_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::PinUpdate(pin_update) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvPinUpdate::new(pin_update))).map_err(|e| e).unwrap();
                            Self::call_event(pin_update_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::MuteUpdate(mute_update) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvMuteUpdate::from(mute_update))).map_err(|e| e).unwrap();
                            Self::call_event(mute_update_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::ArchiveUpdate(archived) => {
                            let payload = Python::attach(|py| {
                                pyo3::Py::new(
                                    py,
                                    EvArchiveUpdate::new(
                                        archived.jid.into(),
                                        archived.timestamp,
                                        Arc::from(archived.action.clone()),
                                        archived.from_full_sync,
                                    ),
                                )
                            })
                            .map_err(|e| e)
                            .unwrap();
                            Self::call_event(archive_update_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::MarkChatAsReadUpdate(mark_chat_as_read_update) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvMarkChatAsReadUpdate::from(mark_chat_as_read_update)))
                                .map_err(|e| e)
                                .unwrap();
                            Self::call_event(mark_chat_as_read_update_callbacks, payload, locals.clone())
                                .await
                                .unwrap();
                        }
                        Event::HistorySync(history_sync) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvHistorySync::from(history_sync))).map_err(|e| e).unwrap();
                            Self::call_event(history_sync_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::OfflineSyncPreview(offline_sync_preview) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvOfflineSyncPreview::from(offline_sync_preview))).map_err(|e| e).unwrap();
                            Self::call_event(offline_sync_preview_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::OfflineSyncCompleted(offline_sync_complete) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvOfflineSyncCompleted::from(offline_sync_complete)))
                                .map_err(|e| e)
                                .unwrap();
                            Self::call_event(offline_sync_completed_callbacks, payload, locals.clone())
                                .await
                                .unwrap();
                        }
                        Event::DeviceListUpdate(device_list_update) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvDeviceListUpdate::from(device_list_update))).map_err(|e| e).unwrap();
                            Self::call_event(device_list_update_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::BusinessStatusUpdate(_) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvBusinessStatusUpdate {}))
                                .map_err(|e| e)
                                .unwrap();
                            Self::call_event(business_status_update_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::StreamReplaced(_) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvStreamReplaced {})).map_err(|e| e).unwrap();
                            Self::call_event(stream_replaced_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::TemporaryBan(temporary_ban) => {
                            let payload = Python::attach(|py| {
                                pyo3::Py::new(py, EvTemporaryBan::from_wacore(temporary_ban))
                            })
                            .map_err(|e| e)
                            .unwrap();
                            Self::call_event(temporary_ban_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::ConnectFailure(connect_failure) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvConnectFailure::new(connect_failure.reason, connect_failure.message, connect_failure.raw))).map_err(|e| e).unwrap();
                            Self::call_event(connect_failure_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::StreamError(stream_error) => {
                            let payload = Python::attach(|py| {
                                Py::new(py, EvStreamError::new(
                                    stream_error.code,
                                    stream_error.raw
                                ))
                            }).unwrap();
                            Self::call_event(stream_error_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::ContactNumberChanged(contact_number_changed) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvContactNumberChanged::new(contact_number_changed.old_jid.into(), contact_number_changed.new_jid.into(), contact_number_changed.old_lid.map(|f|f.into()), contact_number_changed.new_lid.map(|f|f.into()), contact_number_changed.timestamp))).unwrap();
                            Self::call_event(contact_number_changed_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::ContactSyncRequested(contact_sync_requested) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvContactSyncRequested::new(contact_sync_requested.after, contact_sync_requested.timestamp))).unwrap();
                            Self::call_event(contact_sync_requested_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::ContactUpdated(contact_updated) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvContactUpdated::new(contact_updated.jid, contact_updated.timestamp))).unwrap();
                            Self::call_event(contact_updated_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::StarUpdate(star_update) => {
                            let payload = Python::attach(|py| pyo3::Py::new(py, EvStarUpdate::new(star_update.chat_jid, star_update.participant_jid, star_update.message_id, star_update.from_me, star_update.timestamp, star_update.from_full_sync, star_update.action.starred)  )).unwrap();
                            Self::call_event(star_update_callbacks, payload, locals.clone()).await.unwrap();
                        }
                        Event::DisappearingModeChanged(disappearing_mode_changed) => {
                            let payload = Python::attach(|py| {
                                pyo3::Py::new(py, EvDisappearingModeChanged::new(
                                    disappearing_mode_changed.from,
                                    disappearing_mode_changed.duration,
                                    disappearing_mode_changed.setting_timestamp
                                ))
                            }).unwrap();
                            Self::call_event(disappearing_mode_changed_callbacks, payload, locals.clone()).await.unwrap();
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