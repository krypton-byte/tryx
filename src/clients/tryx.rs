use std::sync::{Arc};
use std::future::Future;
use std::pin::Pin;
use pyo3::{Bound, PyAny, pyclass, pymethods};
use pyo3::prelude::*;
use pyo3_async_runtimes::{TaskLocals, into_future_with_locals};
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals, into_future};
use tokio::sync::watch;
use tokio::time::{Duration, interval};
use wacore::types::events::Event;
use whatsapp_rust::{CacheConfig, Client, TokioRuntime};
use whatsapp_rust::bot::Bot;
use whatsapp_rust::store::Backend;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;
use tokio::signal;
use tracing::{debug, error, info, warn};
use super::community::CommunityClient;
use super::advanced::AdvancedClient;
use super::comments::CommentsClient;
use super::contacts::ContactClient;
use super::chatstate::ChatstateClient;
use super::events::EventsClient;
use super::labels::LabelsClient;
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
use crate::backend::SqliteStore;
use crate::events::types::{
    EvAppStateSyncFailed, EvArchiveUpdate, EvBusinessStatusUpdate, EvChatPresence, EvClientExpirationChanged, EvClientOutDated, EvConnectFailure, EvConnected, EvContactNumberChanged, EvContactRemoved, EvContactSyncRequested, EvContactUpdate, EvContactUpdated, EvDeleteChatUpdate, EvDeleteMessageForMeUpdate, EvDeviceListUpdate, EvDisableLinkPreviewsUpdate, EvDisappearingModeChanged, EvDisconnected, EvFavoritesUpdate, EvGroupUpdate, EvHistorySync, EvIdentityChange, EvLockChatUpdate, EvLoggedOut, EvMarkChatAsReadUpdate, EvMessage, EvMexNotification, EvMuteUpdate, EvNewsletterLiveUpdate, EvNotification, EvOfflineSyncCompleted, EvOfflineSyncInterrupted, EvOfflineSyncPreview, EvPairError, EvPairSuccess, EvPairingCode, EvPairingQrCode, EvPictureUpdate, EvPinUpdate, EvPresence, EvPushNameUpdate, EvQrScannedWithoutMultidevice, EvReceipt, EvSelfPushNameUpdated, EvServerAck, EvStarUpdate, EvStatusPrivacyUpdate, EvStreamError, EvStreamReplaced, EvTemporaryBan, EvUnarchiveChatsSettingUpdate, EvUndecryptableMessage, EvUserAboutUpdate
};
use crate::exceptions::{EventDispatchError, FailedBuildClient, UnsupportedBackend};
use crate::events::dispatcher::Dispatcher;
use super::event_callbacks::EventCallbacks;
use super::voip::VoipClient;

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
    signal_cache_max_entries: Option<usize>,
}

impl Tryx {
    fn build_tryx_client(
        py: Python<'_>,
        client_rx: watch::Receiver<Option<Arc<Client>>>,
    ) -> PyResult<Py<TryxClient>> {
        Py::new(
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
                advanced: new_namespace_client!(py, client_rx, AdvancedClient),
                labels: new_namespace_client!(py, client_rx, LabelsClient),
                comments: new_namespace_client!(py, client_rx, CommentsClient),
                events: new_namespace_client!(py, client_rx, EventsClient),
                voip: new_namespace_client!(py, client_rx, VoipClient),
            },
        )
    }
}

#[pymethods]
impl Tryx {
    #[new]
    #[pyo3(signature = (backend, signal_cache_max_entries=None))]
    fn new(py: Python, backend: Py<PyAny>, signal_cache_max_entries: Option<usize>) -> PyResult<Self> {
        init_logging();
        info!("initializing Tryx client");
        if let Ok(sqlite) = backend.extract::<Py<SqliteStore>>(py) {
            debug!("detected sqlite backend from Python");
            let fut = sqlite.borrow(py).connect_future();
            let rt = pyo3_async_runtimes::tokio::get_runtime();
            let store = py
                .detach(|| rt.block_on(fut))
                .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)?;
            let (client_tx, client_rx) = watch::channel(None);
            let tryx_client = Self::build_tryx_client(py, client_rx)?;

            info!("backend connected and dispatcher initialized");
            Ok(Tryx {
                backend: Arc::new(store),
                handlers: Py::new(py, Dispatcher::empty())?,
                tryx_client,
                client_tx,
                signal_cache_max_entries,
            })
        } else if let (Ok(lib_path), Ok(config_json)) = (
            backend.getattr(py, "lib_path").and_then(|v| v.extract::<String>(py)),
            backend.getattr(py, "config_json").and_then(|v| v.extract::<String>(py)),
        ) {
            debug!("detected FFI backend from Python via duck-typing");

            let rt = pyo3_async_runtimes::tokio::get_runtime();
            let ffi_store = py
                .detach(|| {
                    rt.block_on(crate::backend::ffi_bridge::FfiBridgeStore::connect(
                        &lib_path,
                        &config_json,
                    ))
                })
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let (client_tx, client_rx) = watch::channel(None);
            let tryx_client = Self::build_tryx_client(py, client_rx)?;

            info!("ffi backend connected and dispatcher initialized");
            Ok(Tryx {
                backend: Arc::new(ffi_store),
                handlers: Py::new(py, Dispatcher::empty())?,
                tryx_client,
                client_tx,
                signal_cache_max_entries,
            })
        } else if backend.getattr(py, "put_identity").is_ok() {
            debug!("detected python store backend via duck-typing");

            let python_store = crate::backend::python_store::PythonStore::new(backend.clone_ref(py));

            let (client_tx, client_rx) = watch::channel(None);
            let tryx_client = Self::build_tryx_client(py, client_rx)?;

            info!("python backend connected and dispatcher initialized");
            Ok(Tryx {
                backend: Arc::new(python_store),
                handlers: Py::new(py, Dispatcher::empty())?,
                tryx_client,
                client_tx,
                signal_cache_max_entries,
            })
        } else {
            error!("unsupported backend type passed to Tryx");
            Err(PyErr::new::<UnsupportedBackend, _>("Unsupported backend type"))
        }
    }
    fn get_client(&self, py: Python<'_>) -> Py<TryxClient> {
        self.tryx_client.clone_ref(py)
    }
    /// Returns a stateless decorator compatible with:
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

    fn shutdown<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client_opt = self.client_tx.borrow().clone();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            if let Some(client) = client_opt {
                let _ = client.shutdown().await;
            }
            Ok(())
        })
    }

    fn run<'py>(&'py self, py: Python<'py>) -> Result<Bound<'py, PyAny>, PyErr> {
        init_logging();
        info!("starting client in async mode via Tryx.run");
        let backend = self.backend.clone();
        let handlers = self.handlers.clone_ref(py);
        let tryx_client = self.tryx_client.clone_ref(py);
        let client_tx = self.client_tx.clone();
        let signal_cache_max_entries = self.signal_cache_max_entries;
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals.clone(), async move {
            Self::run_automation(
                backend,
                handlers,
                Some(locals),
                tryx_client,
                client_tx,
                signal_cache_max_entries,
            )
            .await
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
        let client_rx = self.client_tx.subscribe();
        let signal_cache_max_entries = self.signal_cache_max_entries;
        py.detach(move || {
            let rt = pyo3_async_runtimes::tokio::get_runtime();
            rt.block_on(async {
                let mut task = tokio::spawn(Self::run_automation(
                    backend,
                    handlers,
                    None,
                    tryx_client,
                    client_tx,
                    signal_cache_max_entries,
                ));
                let mut signal_tick = interval(Duration::from_millis(200));
                let mut non_kb_err: Option<PyErr> = None;

                loop {
                    tokio::select! {
                        _ = signal::ctrl_c() => {
                            warn!("SIGINT received via tokio::signal, initiating graceful shutdown");
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
                                    warn!("KeyboardInterrupt detected from Python, initiating graceful shutdown");
                                    break;
                                }

                                error!(error = %err, "non-keyboard Python signal error while polling");
                                non_kb_err = Some(err);
                                break;
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

                // Gracefully shut down the WhatsApp client so Signal ratchet state flushes cleanly.
                let active_client = client_rx.borrow().clone();
                if let Some(client) = active_client {
                    let _ = tokio::time::timeout(Duration::from_secs(5), client.shutdown()).await;
                }
                match tokio::time::timeout(Duration::from_secs(5), &mut task).await {
                    Ok(Ok(Ok(()))) => info!("client finished gracefully after interrupt"),
                    Ok(Ok(Err(err))) => {
                        if let Some(py_err) = non_kb_err {
                            return Err(py_err);
                        }
                        return Err(err);
                    }
                    Ok(Err(join_err)) if join_err.is_cancelled() => info!("task cancelled successfully"),
                    Ok(Err(join_err)) => {
                        error!(error = %join_err, "task join failed after interrupt");
                        return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(join_err.to_string()));
                    }
                    Err(_) => {
                        warn!("graceful shutdown timed out, aborting run task");
                        task.abort();
                        let _ = task.await;
                    }
                }

                if let Some(py_err) = non_kb_err {
                    return Err(py_err);
                }

                info!("blocking run interrupted and finished");
                Ok(())
            })
        })
    }
}
    

impl Tryx {
    /// Build the Python event payload and invoke/schedule all callbacks within a single
    /// Python GIL acquisition. Supports both `async def` and synchronous `def` handlers.
    async fn emit_built_event<F>(
        tryx_client: &Arc<Py<TryxClient>>,
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
        let scheduled = Python::attach(|py| -> PyResult<Vec<PyCallbackFuture>> {
            let payload = build_payload(py)?;
            let client_ref = tryx_client.clone_ref(py);
            let mut futures = Vec::new();
            for callback in callbacks {
                debug!(event = event_name, "calling Python event callback");
                let result = callback
                    .bind(py)
                    .call1((client_ref.clone_ref(py), payload.clone_ref(py)))?;
                if result.hasattr("__await__")? {
                    let fut: PyCallbackFuture = match locals.as_ref() {
                        Some(locals) => Box::pin(into_future_with_locals(locals, result)?),
                        None => Box::pin(into_future(result)?),
                    };
                    futures.push(fut);
                }
            }
            Ok(futures)
        });

        match scheduled {
            Ok(futures) => {
                for py_future in futures {
                    if let Err(err) = py_future.await {
                        error!(event = event_name, error = %err, "event callback failed");
                        Python::attach(|py| err.print(py));
                    }
                }
            }
            Err(err) => {
                error!(event = event_name, error = %err, "failed to dispatch python event");
                Python::attach(|py| err.print(py));
            }
        }
    }

    async fn run_automation(
        backend: Arc<dyn Backend>,
        handlers: Py<Dispatcher>,
        locals: Option<TaskLocals>,
        tryx_client: Py<TryxClient>,
        client_tx: watch::Sender<Option<Arc<Client>>>,
        signal_cache_max_entries: Option<usize>,
    ) -> PyResult<()> {
        let callbacks = Arc::new(Python::attach(|py| {
            let dispatcher = handlers.bind(py).borrow();
            EventCallbacks::from_dispatcher(py, &dispatcher)
        }));
        let interests = callbacks.event_interests();
        let tryx_client = Arc::new(tryx_client);
        info!(subscribed_events = interests.len(), "building WhatsApp automation client");
        let mut builder = Bot::builder()
            .with_backend_arc(backend)
            .with_transport_factory(TokioWebSocketTransportFactory::new())
            .with_http_client(UreqHttpClient::new());
        if let Some(cap) = signal_cache_max_entries {
            let mut cache_cfg = CacheConfig::default();
            cache_cfg.signal_cache_max_entries = Some(cap);
            builder = builder.with_cache_config(cache_cfg);
        }
        let automation = builder
            .on_event_for(&interests, move |event, client| {
                let locals = locals.clone();
                let callbacks = Arc::clone(&callbacks);
                let tryx_client = Arc::clone(&tryx_client);

                async move {
                    match event.as_ref() {
                        Event::IncomingCall(incoming) => {
                            if !callbacks.incoming_call.is_empty()
                                && matches!(incoming.action, whatsapp_rust::types::call::CallAction::Offer { .. })
                            {
                                let incoming = (**incoming).clone();
                                let client = client.clone();
                                Self::emit_built_event(&tryx_client, &callbacks.incoming_call, locals.clone(), "IncomingCall", |py| {
                                    Py::new(py, crate::clients::voip::IncomingCallEvent {
                                        client,
                                        incoming: std::sync::Mutex::new(Some(incoming)),
                                    }).map(|event| event.into_any())
                                }).await;
                            }
                        }
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
                            Self::emit_built_event(&tryx_client, &callbacks.logout, locals.clone(), "LoggedOut", |py| {
                                Py::new(py, EvLoggedOut::new((**logout).clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::PairSuccess(pair_success) => {
                            Self::emit_built_event(&tryx_client, &callbacks.pair_success, locals.clone(), "PairSuccess", |py| {
                                Py::new(py, EvPairSuccess::from(pair_success.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::PairError(pair_error) => {
                            Self::emit_built_event(
                                &tryx_client,
                                &callbacks.pair_error,
                                locals.clone(),
                                "PairError",
                                |py| {
                                    let ev = EvPairError::new(
                                        py,
                                        pair_error.id.clone().into(),
                                        pair_error.lid.clone().into(),
                                        pair_error.business_name.clone(),
                                        pair_error.platform.clone(),
                                        pair_error.error.clone(),
                                    )?;
                                    Py::new(py, ev).map(|event| event.into_any())
                                },
                            )
                            .await;
                        }
                        Event::PairingQrCode(qr) => {
                            Self::emit_built_event(&tryx_client, &callbacks.pairing_qr, locals.clone(), "PairingQrCode", |py| {
                                Py::new(py, EvPairingQrCode::new(qr.code.clone(), qr.timeout.as_secs())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::PairingCode(pairing_code) => {
                            Self::emit_built_event(&tryx_client, &callbacks.pairing_code, locals.clone(), "PairingCode", |py| {
                                Py::new(py, EvPairingCode::new(pairing_code.code.clone(), pairing_code.timeout.as_secs())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::QrScannedWithoutMultidevice(scanned) => {
                            Self::emit_built_event(&tryx_client, &callbacks.qr_scanned_without_multidevice, locals.clone(), "QrScannedWithoutMultidevice", |py| {
                                Py::new(py, EvQrScannedWithoutMultidevice::from(scanned.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ClientOutdated(_) => {
                            Self::emit_built_event(&tryx_client, &callbacks.client_outdated, locals.clone(), "ClientOutdated", |py| {
                                Py::new(py, EvClientOutDated {}).map(|event| event.into_any())
                            }).await;
                        }
                        Event::Messages(batch) => {
                            if callbacks.message.is_empty() {
                                return;
                            }
                            for inbound in batch.messages.iter() {
                                Self::emit_built_event(&tryx_client, &callbacks.message, locals.clone(), "Message", |py| {
                                    Py::new(py, EvMessage::from_arcs(Arc::clone(&inbound.message), Arc::clone(&inbound.info)))
                                        .map(|event| event.into_any())
                                }).await;
                            }
                        }
                        Event::Receipt(receipt) => {
                            Self::emit_built_event(
                                &tryx_client,
                                &callbacks.receipt,
                                locals.clone(),
                                "Receipt",
                                |py| {
                                    Ok(EvReceipt::new(
                                        py,
                                        receipt.source.clone(),
                                        receipt.message_ids.iter().map(|id| id.to_string()).collect(),
                                        receipt.timestamp,
                                        receipt.r#type.clone(),
                                    )?
                                    .into_any())
                                },
                            )
                            .await;
                        }
                        Event::UndecryptableMessage(undecryptable_message) => {
                            Self::emit_built_event(&tryx_client, &callbacks.undecryptable_message, locals.clone(), "UndecryptableMessage", |py| {
                                let ev = EvUndecryptableMessage::new(
                                    py,
                                    (*undecryptable_message.info).clone(),
                                    undecryptable_message.is_unavailable,
                                    undecryptable_message.unavailable_type,
                                    undecryptable_message.decrypt_fail_mode,
                                )?;
                                Py::new(py, ev).map(|event| event.into_any())
                            }).await;
                        }
                        Event::Notification(notification) => {
                            Self::emit_built_event(&tryx_client, &callbacks.notification, locals.clone(), "Notification", |py| {
                                Py::new(py, EvNotification::new(notification.to_owned_node())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ChatPresence(chat_presence) => {
                            Self::emit_built_event(&tryx_client, &callbacks.chat_presence, locals.clone(), "ChatPresence", |py| {
                                Py::new(py, EvChatPresence::from(chat_presence.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::Presence(presence) => {
                            Self::emit_built_event(&tryx_client, &callbacks.presence, locals.clone(), "Presence", |py| {
                                Py::new(py, EvPresence::from_event(py, presence)?).map(|event| event.into_any())
                            }).await;
                        }
                        Event::PictureUpdate(picture_update) => {
                            Self::emit_built_event(&tryx_client, &callbacks.picture_update, locals.clone(), "PictureUpdate", |py| {
                                Py::new(py, EvPictureUpdate::new(picture_update.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::UserAboutUpdate(user_about) => {
                            Self::emit_built_event(&tryx_client, &callbacks.user_about_update, locals.clone(), "UserAboutUpdate", |py| {
                                Py::new(py, EvUserAboutUpdate::new(user_about.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        // JoinedGroup is not a current Event variant; reserved for future use.
                        Event::GroupUpdate(group_info) => {
                            Self::emit_built_event(&tryx_client, &callbacks.group_info_update, locals.clone(), "GroupUpdate", |py| {
                                Py::new(py, EvGroupUpdate::new(group_info.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ContactUpdate(contact_update) => {
                            Self::emit_built_event(&tryx_client, &callbacks.contact_update, locals.clone(), "ContactUpdate", |py| {
                                Py::new(py, EvContactUpdate::new(contact_update.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::RetiredPushNameUpdate(pushname) => {
                            Self::emit_built_event(&tryx_client, &callbacks.push_name_update, locals.clone(), "RetiredPushNameUpdate", |py| {
                                Py::new(py, EvPushNameUpdate::from(pushname.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::SelfPushNameUpdated(self_push_name_update) => {
                            Self::emit_built_event(&tryx_client, &callbacks.self_push_name_updated, locals.clone(), "SelfPushNameUpdated", |py| {
                                Py::new(py, EvSelfPushNameUpdated::from(self_push_name_update.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::PinUpdate(pin_update) => {
                            Self::emit_built_event(&tryx_client, &callbacks.pin_update, locals.clone(), "PinUpdate", |py| {
                                Py::new(py, EvPinUpdate::new(pin_update.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::MuteUpdate(mute_update) => {
                            Self::emit_built_event(&tryx_client, &callbacks.mute_update, locals.clone(), "MuteUpdate", |py| {
                                Py::new(py, EvMuteUpdate::from(mute_update.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ArchiveUpdate(archived) => {
                            Self::emit_built_event(&tryx_client, &callbacks.archive_update, locals.clone(), "ArchiveUpdate", |py| {
                                Py::new(py, EvArchiveUpdate::from(archived.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::MarkChatAsReadUpdate(mark_chat_as_read_update) => {
                            Self::emit_built_event(&tryx_client, &callbacks.mark_chat_as_read_update, locals.clone(), "MarkChatAsReadUpdate", |py| {
                                Py::new(py, EvMarkChatAsReadUpdate::from(mark_chat_as_read_update.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::HistorySync(history_sync) => {
                            Self::emit_built_event(&tryx_client, &callbacks.history_sync, locals.clone(), "HistorySync", |py| {
                                Py::new(py, EvHistorySync::from_lazy((**history_sync).clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::OfflineSyncPreview(offline_sync_preview) => {
                            Self::emit_built_event(&tryx_client, &callbacks.offline_sync_preview, locals.clone(), "OfflineSyncPreview", |py| {
                                Py::new(py, EvOfflineSyncPreview::from(offline_sync_preview.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::OfflineSyncCompleted(offline_sync_complete) => {
                            Self::emit_built_event(&tryx_client, &callbacks.offline_sync_completed, locals.clone(), "OfflineSyncCompleted", |py| {
                                Py::new(py, EvOfflineSyncCompleted::from(offline_sync_complete.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::DeviceListUpdate(device_list_update) => {
                            Self::emit_built_event(&tryx_client, &callbacks.device_list_update, locals.clone(), "DeviceListUpdate", |py| {
                                Py::new(py, EvDeviceListUpdate::from(device_list_update.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::BusinessStatusUpdate(business_status_update) => {
                            Self::emit_built_event(&tryx_client, &callbacks.business_status_update, locals.clone(), "BusinessStatusUpdate", |py| {
                                Py::new(py, EvBusinessStatusUpdate::from(business_status_update.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::StreamReplaced(_) => {
                            Self::emit_built_event(&tryx_client, &callbacks.stream_replaced, locals.clone(), "StreamReplaced", |py| {
                                Py::new(py, EvStreamReplaced {}).map(|event| event.into_any())
                            }).await;
                        }
                        Event::TemporaryBan(temporary_ban) => {
                            Self::emit_built_event(&tryx_client, &callbacks.temporary_ban, locals.clone(), "TemporaryBan", |py| {
                                Py::new(py, EvTemporaryBan::from(temporary_ban.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ConnectFailure(connect_failure) => {
                            Self::emit_built_event(&tryx_client, &callbacks.connect_failure, locals.clone(), "ConnectFailure", |py| {
                                Py::new(py, EvConnectFailure::new(connect_failure.reason.clone(), connect_failure.message.clone(), connect_failure.raw.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::StreamError(stream_error) => {
                            Self::emit_built_event(&tryx_client, &callbacks.stream_error, locals.clone(), "StreamError", |py| {
                                Py::new(py, EvStreamError::new(stream_error.code.clone(), stream_error.raw.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ContactNumberChanged(contact_number_changed) => {
                            Self::emit_built_event(&tryx_client, &callbacks.contact_number_changed, locals.clone(), "ContactNumberChanged", |py| {
                                Py::new(py, EvContactNumberChanged::from(contact_number_changed.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ContactSyncRequested(contact_sync_requested) => {
                            Self::emit_built_event(&tryx_client, &callbacks.contact_sync_requested, locals.clone(), "ContactSyncRequested", |py| {
                                Py::new(py, EvContactSyncRequested::from(contact_sync_requested.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ContactUpdated(contact_updated) => {
                            Self::emit_built_event(&tryx_client, &callbacks.contact_updated, locals.clone(), "ContactUpdated", |py| {
                                Py::new(py, EvContactUpdated::from(contact_updated.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::StarUpdate(star_update) => {
                            Self::emit_built_event(&tryx_client, &callbacks.star_update, locals.clone(), "StarUpdate", |py| {
                                Py::new(py, EvStarUpdate::from(star_update.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::DisappearingModeChanged(disappearing_mode_changed) => {
                            Self::emit_built_event(&tryx_client, &callbacks.disappearing_mode_changed, locals.clone(), "DisappearingModeChanged", |py| {
                                Py::new(py, EvDisappearingModeChanged::from(disappearing_mode_changed.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::NewsletterLiveUpdate(newsletter_live_update) => {
                            Self::emit_built_event(&tryx_client, &callbacks.newsletter_live_update, locals.clone(), "NewsletterLiveUpdate", |py| {
                                Py::new(py, EvNewsletterLiveUpdate::from(newsletter_live_update.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::DeleteChatUpdate(delete_chat_update) => {
                            Self::emit_built_event(&tryx_client, &callbacks.delete_chat_update, locals.clone(), "DeleteChatUpdate", |py| {
                                Py::new(py, EvDeleteChatUpdate::from(delete_chat_update.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::DeleteMessageForMeUpdate(delete_message_for_me_update) => {
                            Self::emit_built_event(&tryx_client, &callbacks.delete_message_for_me_update, locals.clone(), "DeleteMessageForMeUpdate", |py| {
                                Py::new(py, EvDeleteMessageForMeUpdate::from(delete_message_for_me_update.clone())).map(|event| event.into_any())
                            }).await;
                        }
                        Event::IdentityChange(ev) => {
                            Self::emit_built_event(&tryx_client, &callbacks.identity_change, locals.clone(), "IdentityChange", |py| {
                                Py::new(py, EvIdentityChange::from_event(py, ev)?).map(|event| event.into_any())
                            }).await;
                        }
                        Event::LockChatUpdate(ev) => {
                            Self::emit_built_event(&tryx_client, &callbacks.lock_chat_update, locals.clone(), "LockChatUpdate", |py| {
                                Py::new(py, EvLockChatUpdate::from_event(py, ev)?).map(|event| event.into_any())
                            }).await;
                        }
                        Event::FavoritesUpdate(ev) => {
                            Self::emit_built_event(&tryx_client, &callbacks.favorites_update, locals.clone(), "FavoritesUpdate", |py| {
                                Py::new(py, EvFavoritesUpdate::from_event(py, ev)?).map(|event| event.into_any())
                            }).await;
                        }
                        Event::StatusPrivacyUpdate(ev) => {
                            Self::emit_built_event(&tryx_client, &callbacks.status_privacy_update, locals.clone(), "StatusPrivacyUpdate", |py| {
                                Py::new(py, EvStatusPrivacyUpdate::from_event(py, ev)?).map(|event| event.into_any())
                            }).await;
                        }
                        Event::UnarchiveChatsSettingUpdate(ev) => {
                            Self::emit_built_event(&tryx_client, &callbacks.unarchive_chats_setting_update, locals.clone(), "UnarchiveChatsSettingUpdate", |py| {
                                Py::new(py, EvUnarchiveChatsSettingUpdate::from_event(py, ev)?).map(|event| event.into_any())
                            }).await;
                        }
                        Event::DisableLinkPreviewsUpdate(ev) => {
                            Self::emit_built_event(&tryx_client, &callbacks.disable_link_previews_update, locals.clone(), "DisableLinkPreviewsUpdate", |py| {
                                Py::new(py, EvDisableLinkPreviewsUpdate::from_event(py, ev)?).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ContactRemoved(ev) => {
                            Self::emit_built_event(&tryx_client, &callbacks.contact_removed, locals.clone(), "ContactRemoved", |py| {
                                Py::new(py, EvContactRemoved::from_event(py, ev)?).map(|event| event.into_any())
                            }).await;
                        }
                        Event::OfflineSyncInterrupted(ev) => {
                            Self::emit_built_event(&tryx_client, &callbacks.offline_sync_interrupted, locals.clone(), "OfflineSyncInterrupted", |py| {
                                Py::new(py, EvOfflineSyncInterrupted::from(ev)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ClientExpirationChanged(ev) => {
                            Self::emit_built_event(&tryx_client, &callbacks.client_expiration_changed, locals.clone(), "ClientExpirationChanged", |py| {
                                Py::new(py, EvClientExpirationChanged::from(ev)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::ServerAck(ev) => {
                            Self::emit_built_event(&tryx_client, &callbacks.server_ack, locals.clone(), "ServerAck", |py| {
                                Py::new(py, EvServerAck::from_event(py, ev)?).map(|event| event.into_any())
                            }).await;
                        }
                        Event::AppStateSyncFailed(ev) => {
                            Self::emit_built_event(&tryx_client, &callbacks.app_state_sync_failed, locals.clone(), "AppStateSyncFailed", |py| {
                                Py::new(py, EvAppStateSyncFailed::from(ev)).map(|event| event.into_any())
                            }).await;
                        }
                        Event::MexNotification(ev) => {
                            Self::emit_built_event(&tryx_client, &callbacks.mex_notification, locals.clone(), "MexNotification", |py| {
                                Py::new(py, EvMexNotification::from_event(py, ev)?).map(|event| event.into_any())
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
        // `Bot::run` now consumes the bot and drives the event loop to
        // completion, returning `()` instead of a fallible stream handle.
        automation.run().await;

        info!("run loop finished");

        Ok(())
    }
}
