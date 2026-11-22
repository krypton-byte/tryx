use std::sync::{Arc};
use pyo3::{Bound, PyAny, pyclass, pymethods, types::PyBytes};
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use waproto::whatsapp::Message as WhatsappMessage;
use waproto::whatsapp::message::{self as wa};
use wacore::proto_helpers::build_quote_context;
use buffa::Message;
use whatsapp_rust::{Client, UploadOptions, download::DownloadLimits};
use crate::clients::advanced::AdvancedClient;
use crate::clients::chat_actions::ChatActionsClient;
use crate::clients::chatstate::ChatstateClient;
use crate::clients::comments::CommentsClient;
use crate::clients::community::CommunityClient;
use crate::clients::contacts::ContactClient;
use crate::clients::events::EventsClient;
use crate::clients::voip::VoipClient;
use crate::clients::labels::LabelsClient;
use crate::clients::blocking::BlockingClient;
use crate::clients::groups::GroupsClient;
use crate::clients::newsletter::NewsletterClient;
use crate::clients::polls::PollsClient;
use crate::clients::presence::PresenceClient;
use crate::clients::privacy::PrivacyClient;
use crate::clients::profile::ProfileClient;
use crate::clients::status::StatusClient;
use crate::events::proto_cache::decode_py_proto;
use crate::events::types::{EvMessage};
use crate::types::{JID, MediaReuploadResult, PyJidArg, SendResult, UploadResponse};
use crate::wacore::download::MediaType;

/// Guess the MIME type from the raw data bytes using magic number signatures.
/// Returns the detected MIME string or `None` if unrecognized.
fn guess_mimetype(data: &[u8]) -> Option<String> {
    infer::get(data).map(|kind| kind.mime_type().to_string())
}

/// Helper to download an `Arc<wa::Message>` directly without Python protobuf round-trip.
async fn download_from_wa_message(
    client: &Client,
    msg: &WhatsappMessage,
    limits: DownloadLimits,
) -> Result<Vec<u8>, PyErr> {
    if let Some(img) = msg.image_message.as_option() {
        client.download(img, limits).await
    } else if let Some(vid) = msg.video_message.as_option() {
        client.download(vid, limits).await
    } else if let Some(doc) = msg.document_message.as_option() {
        client.download(doc, limits).await
    } else if let Some(aud) = msg.audio_message.as_option() {
        client.download(aud, limits).await
    } else if let Some(stk) = msg.sticker_message.as_option() {
        client.download(stk, limits).await
    } else {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Message does not contain downloadable media (image, video, document, audio, or sticker)",
        ));
    }
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Helper to stream an `Arc<wa::Message>` directly into a file without buffering in RAM.
async fn download_wa_message_to_file(
    client: &Client,
    msg: &WhatsappMessage,
    writer: std::io::BufWriter<std::fs::File>,
    limits: DownloadLimits,
) -> Result<(), PyErr> {
    let res = if let Some(img) = msg.image_message.as_option() {
        client.download_to_writer(img, writer, limits).await
    } else if let Some(vid) = msg.video_message.as_option() {
        client.download_to_writer(vid, writer, limits).await
    } else if let Some(doc) = msg.document_message.as_option() {
        client.download_to_writer(doc, writer, limits).await
    } else if let Some(aud) = msg.audio_message.as_option() {
        client.download_to_writer(aud, writer, limits).await
    } else if let Some(stk) = msg.sticker_message.as_option() {
        client.download_to_writer(stk, writer, limits).await
    } else {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Message does not contain downloadable media (image, video, document, audio, or sticker)",
        ));
    };
    res.map(|_| ()).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

/// Reduces the repeated decode-then-download pattern for each media type.
macro_rules! decode_and_download {
    ($client:expr, $data:expr, $msg_type:ty, $label:expr, $limits:expr) => {{
        let media = <$msg_type>::decode(&mut $data).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to decode {}: {}", $label, e),
            )
        })?;
        $client.download(&media, $limits).await
    }};
}

macro_rules! decode_and_download_to_writer {
    ($client:expr, $data:expr, $msg_type:ty, $label:expr, $writer:expr, $limits:expr) => {{
        let media = <$msg_type>::decode(&mut $data).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Failed to decode {}: {}", $label, e),
            )
        })?;
        $client.download_to_writer(&media, $writer, $limits).await
    }};
}
#[pyclass]
pub struct TryxClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
    #[pyo3(get)]
    pub contact: Py<ContactClient>,
    #[pyo3(get)]
    pub chat_actions: Py<ChatActionsClient>,
    #[pyo3(get)]
    pub community: Py<CommunityClient>,
    #[pyo3(get)]
    pub newsletter: Py<NewsletterClient>,
    #[pyo3(get)]
    pub groups: Py<GroupsClient>,
    #[pyo3(get)]
    pub status: Py<StatusClient>,
    #[pyo3(get)]
    pub chatstate: Py<ChatstateClient>,
    #[pyo3(get)]
    pub blocking: Py<BlockingClient>,
    #[pyo3(get)]
    pub polls: Py<PollsClient>,
    #[pyo3(get)]
    pub presence: Py<PresenceClient>,
    #[pyo3(get)]
    pub privacy: Py<PrivacyClient>,
    #[pyo3(get)]
    pub profile: Py<ProfileClient>,
    #[pyo3(get)]
    pub advanced: Py<AdvancedClient>,
    #[pyo3(get)]
    pub labels: Py<LabelsClient>,
    #[pyo3(get)]
    pub comments: Py<CommentsClient>,
    #[pyo3(get)]
    pub events: Py<EventsClient>,
    #[pyo3(get)]
    pub voip: Py<VoipClient>,
}

impl TryxClient {
    fn quote_context(py: Python<'_>, quoted: Option<&Py<EvMessage>>) -> Option<waproto::whatsapp::ContextInfo> {
        quoted.map(|q| {
            let quote = q.bind(py).borrow();
            let msg = quote.inner.as_ref();
            build_quote_context(
                quote.inner_message_info.id.clone(),
                quote.inner_message_info.source.chat.clone(),
                msg,
            )
        })
    }

    fn into_send_result(py: Python<'_>, result: whatsapp_rust::SendResult) -> PyResult<Py<SendResult>> {
        let to = Py::new(py, JID::from(result.to))?;
        Py::new(
            py,
            SendResult {
                message_id: result.message_id.into_string(),
                to,
            },
        )
    }

    fn send_text_inner<'py>(
        &self,
        py: Python<'py>,
        jid: whatsapp_rust::Jid,
        text: String,
        quoted: Option<Py<EvMessage>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first.")
        })?;
        let locals = get_current_locals(py)?;
        let context_info = Self::quote_context(py, quoted.as_ref());
        future_into_py_with_locals::<_, Py<SendResult>>(py, locals, async move {
            match quoted {
                Some(_) => {
                    let mut ext = wa::ExtendedTextMessage::default();
                    ext.text = Some(text);
                    ext.context_info = context_info.into();
                    let mut message = WhatsappMessage::default();
                    message.extended_text_message = buffa::MessageField::some(ext);
                    let send_result = client
                        .send_message(jid, message)
                        .await
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
                    Python::attach(|py| Self::into_send_result(py, send_result))
                }
                None => {
                    let mut message = WhatsappMessage::default();
                    message.conversation = Some(text);
                    let send_result = client
                        .send_message(jid, message)
                        .await
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
                    Python::attach(|py| Self::into_send_result(py, send_result))
                }
            }
        })
    }
}

#[pymethods]
impl TryxClient {
    fn is_connected(&self) -> bool {
        self.client_rx.borrow().is_some()
    }
    /// Download media into memory (`bytes`). Accepts either an `EvMessage` (zero
    /// protobuf round-trip) or a Python protobuf media object (`ImageMessage`, etc.).
    #[pyo3(signature = (target, max_bytes=None))]
    fn download_media<'py>(
        &self,
        py: Python<'py>,
        target: Py<PyAny>,
        max_bytes: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first.")
        })?;
        let limits = match max_bytes {
            Some(limit) => DownloadLimits::new(limit, limit),
            None => DownloadLimits::default(),
        };
        let locals = get_current_locals(py)?;

        // Fast path: EvMessage passed directly -> zero protobuf serialization round-trip!
        if let Ok(ev_msg) = target.extract::<Py<EvMessage>>(py) {
            let msg_arc = Arc::clone(&ev_msg.borrow(py).inner);
            return future_into_py_with_locals::<_, Py<PyBytes>>(py, locals, async move {
                let bytes = download_from_wa_message(&client, &msg_arc, limits).await?;
                Ok(Python::attach(|py| PyBytes::new(py, &bytes).unbind()))
            });
        }

        let message_type_name = target
            .getattr(py, "DESCRIPTOR")
            .and_then(|descriptor| descriptor.getattr(py, "name"))
            .and_then(|name| name.extract::<String>(py))
            .unwrap_or_default();
        let serialized_obj = target.call_method0(py, "SerializeToString")?;
        let serialized = serialized_obj.bind(py).cast::<PyBytes>()?.as_bytes().to_vec();

        future_into_py_with_locals::<_, Py<PyBytes>>(py, locals, async move {
            let download = match message_type_name.as_str() {
                "ImageMessage" => decode_and_download!(client, serialized.as_slice(), wa::ImageMessage, "ImageMessage", limits),
                "VideoMessage" => decode_and_download!(client, serialized.as_slice(), wa::VideoMessage, "VideoMessage", limits),
                "DocumentMessage" => decode_and_download!(client, serialized.as_slice(), wa::DocumentMessage, "DocumentMessage", limits),
                "AudioMessage" => decode_and_download!(client, serialized.as_slice(), wa::AudioMessage, "AudioMessage", limits),
                "StickerMessage" => decode_and_download!(client, serialized.as_slice(), wa::StickerMessage, "StickerMessage", limits),
                _ => {
                    if let Ok(media) = wa::ImageMessage::decode(&mut serialized.as_slice()) {
                        client.download(&media, limits).await
                    } else if let Ok(media) = wa::VideoMessage::decode(&mut serialized.as_slice()) {
                        client.download(&media, limits).await
                    } else if let Ok(media) = wa::DocumentMessage::decode(&mut serialized.as_slice()) {
                        client.download(&media, limits).await
                    } else if let Ok(media) = wa::AudioMessage::decode(&mut serialized.as_slice()) {
                        client.download(&media, limits).await
                    } else if let Ok(media) = wa::StickerMessage::decode(&mut serialized.as_slice()) {
                        client.download(&media, limits).await
                    } else {
                        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "Failed to decode message as supported media type. Expected EvMessage or one of: ImageMessage, VideoMessage, DocumentMessage, AudioMessage, StickerMessage",
                        ));
                    }
                }
            };

            let bytes = download.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(Python::attach(|py| PyBytes::new(py, &bytes).unbind()))
        })
    }

    /// Stream-download and decrypt media directly to a file on disk with constant RAM usage.
    #[pyo3(signature = (target, path, max_bytes=None))]
    fn download_media_to_file<'py>(
        &self,
        py: Python<'py>,
        target: Py<PyAny>,
        path: String,
        max_bytes: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first.")
        })?;
        let limits = match max_bytes {
            Some(limit) => DownloadLimits::new(limit, limit),
            None => DownloadLimits::default(),
        };
        let locals = get_current_locals(py)?;

        if let Ok(ev_msg) = target.extract::<Py<EvMessage>>(py) {
            let msg_arc = Arc::clone(&ev_msg.borrow(py).inner);
            return future_into_py_with_locals(py, locals, async move {
                let file = std::fs::File::create(&path)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
                let writer = std::io::BufWriter::new(file);
                download_wa_message_to_file(&client, &msg_arc, writer, limits).await
            });
        }

        let message_type_name = target
            .getattr(py, "DESCRIPTOR")
            .and_then(|descriptor| descriptor.getattr(py, "name"))
            .and_then(|name| name.extract::<String>(py))
            .unwrap_or_default();
        let serialized_obj = target.call_method0(py, "SerializeToString")?;
        let serialized = serialized_obj.bind(py).cast::<PyBytes>()?.as_bytes().to_vec();

        future_into_py_with_locals(py, locals, async move {
            let file = std::fs::File::create(&path)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
            let writer = std::io::BufWriter::new(file);
            let res = match message_type_name.as_str() {
                "ImageMessage" => decode_and_download_to_writer!(client, serialized.as_slice(), wa::ImageMessage, "ImageMessage", writer, limits),
                "VideoMessage" => decode_and_download_to_writer!(client, serialized.as_slice(), wa::VideoMessage, "VideoMessage", writer, limits),
                "DocumentMessage" => decode_and_download_to_writer!(client, serialized.as_slice(), wa::DocumentMessage, "DocumentMessage", writer, limits),
                "AudioMessage" => decode_and_download_to_writer!(client, serialized.as_slice(), wa::AudioMessage, "AudioMessage", writer, limits),
                "StickerMessage" => decode_and_download_to_writer!(client, serialized.as_slice(), wa::StickerMessage, "StickerMessage", writer, limits),
                _ => {
                    if let Ok(media) = wa::ImageMessage::decode(&mut serialized.as_slice()) {
                        client.download_to_writer(&media, writer, limits).await
                    } else if let Ok(media) = wa::VideoMessage::decode(&mut serialized.as_slice()) {
                        client.download_to_writer(&media, writer, limits).await
                    } else if let Ok(media) = wa::DocumentMessage::decode(&mut serialized.as_slice()) {
                        client.download_to_writer(&media, writer, limits).await
                    } else if let Ok(media) = wa::AudioMessage::decode(&mut serialized.as_slice()) {
                        client.download_to_writer(&media, writer, limits).await
                    } else if let Ok(media) = wa::StickerMessage::decode(&mut serialized.as_slice()) {
                        client.download_to_writer(&media, writer, limits).await
                    } else {
                        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "Failed to decode message as supported media type.",
                        ));
                    }
                }
            };
            res.map(|_| ()).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn upload_file<'py>(&self, py: Python<'py>, path: String, media_type: Py<MediaType>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first.")
        })?;
        let media_type_enum = media_type.bind(py).borrow_mut().to_wacore_enum();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals(py, locals, async move {
            let data = tokio::task::spawn_blocking(move || std::fs::read(&path))
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
            let url = client
                .upload(data, media_type_enum, UploadOptions::default())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(UploadResponse { inner: url })
        })
    }
    fn upload<'py>(&self, py: Python<'py>, data: Vec<u8>, media_type: Py<MediaType>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first.")
        })?;
        let mtype = media_type.bind(py).borrow_mut().to_wacore_enum();
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, UploadResponse>(py, locals, async move {
            let url = client
                .upload(data, mtype, UploadOptions::default())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            let result = UploadResponse {
                inner: url,
            };
            Ok(result)
        })
        //     Ok(url)
        // })
    }
    fn send_message<'py>(&self, py: Python<'py>, to: PyJidArg<'py>, message: Py<PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first.")
        })?;

        let jid = to.into_whatsapp_jid()?;
        let whatsapp_message: WhatsappMessage = decode_py_proto(py, &message)?;

        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, Py<SendResult>>(py, locals, async move {
            let send_result = client
                .send_message(jid, whatsapp_message)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| Self::into_send_result(py, send_result))
        })
    }
    #[pyo3(signature = (to, text, quoted=None))]
    fn send_text<'py>(&self, py: Python<'py>, to: PyJidArg<'py>, text: String, quoted: Option<Py<EvMessage>>) -> PyResult<Bound<'py, PyAny>> {
        let jid = to.into_whatsapp_jid()?;
        self.send_text_inner(py, jid, text, quoted)
    }

    /// Reply directly to an incoming `EvMessage` with text, automatically quoting it.
    fn reply_text<'py>(&self, py: Python<'py>, event: Py<EvMessage>, text: String) -> PyResult<Bound<'py, PyAny>> {
        let chat = event.bind(py).borrow().inner_message_info.source.chat.clone();
        self.send_text_inner(py, chat, text, Some(event))
    }

    /// React directly to an incoming `EvMessage` with an emoji (pass `""` to remove reaction).
    fn react<'py>(&self, py: Python<'py>, event: Py<EvMessage>, emoji: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first.")
        })?;
        let ev_ref = event.bind(py).borrow();
        let info = &ev_ref.inner_message_info;
        let chat_jid_value = info.source.chat.clone();
        let mut key = waproto::whatsapp::MessageKey::default();
        key.remote_jid = Some(chat_jid_value.to_string());
        key.id = Some(info.id.to_string());
        key.from_me = Some(info.source.is_from_me);
        if info.source.is_group {
            key.participant = Some(info.source.sender.to_string());
        }
        drop(ev_ref);
        let mut react_msg = wa::ReactionMessage::default();
        react_msg.key = buffa::MessageField::some(key);
        react_msg.text = Some(emoji);
        react_msg.sender_timestamp_ms = Some(wacore::time::now_millis());
        let mut message = WhatsappMessage::default();
        message.reaction_message = buffa::MessageField::some(react_msg);
        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, Py<SendResult>>(py, locals, async move {
            let send_result = client
                .send_message(chat_jid_value, message)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| Self::into_send_result(py, send_result))
        })
    }

    #[pyo3(signature = (to, photo_data, mimetype=None, caption=None, quoted=None))]
    fn send_photo<'py>(&self, py: Python<'py>, to: PyJidArg<'py>, photo_data: Vec<u8>, mimetype: Option<String>, caption: Option<String>, quoted: Option<Py<EvMessage>>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first.")
        })?;
        let jid = to.into_whatsapp_jid()?;
        let locals = get_current_locals(py)?;
        let context_info = Self::quote_context(py, quoted.as_ref());
        future_into_py_with_locals::<_, Py<SendResult>>(py, locals, async move {
            let mime = mimetype.unwrap_or_else(|| {
                guess_mimetype(&photo_data).unwrap_or_else(|| "image/jpeg".to_string())
            });
            let upload = client
                .upload(photo_data, wacore::download::MediaType::Image, UploadOptions::default())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            let mut img = wa::ImageMessage::default();
            img.url = Some(upload.url);
            img.mimetype = Some(mime);
            img.direct_path = Some(upload.direct_path);
            img.media_key = Some(upload.media_key.to_vec());
            img.file_enc_sha256 = Some(upload.file_enc_sha256.to_vec());
            img.file_sha256 = Some(upload.file_sha256.to_vec());
            img.file_length = Some(upload.file_length);
            img.caption = caption;
            img.context_info = context_info.into();
            let mut message = WhatsappMessage::default();
            message.image_message = buffa::MessageField::some(img);
            let send_result = client
                .send_message(jid, message)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| Self::into_send_result(py, send_result))
        })
    }

    #[pyo3(signature = (to, document_data, mimetype=None, file_name=None, caption=None, quoted=None))]
    fn send_document<'py>(
        &self,
        py: Python<'py>,
        to: PyJidArg<'py>,
        document_data: Vec<u8>,
        mimetype: Option<String>,
        file_name: Option<String>,
        caption: Option<String>,
        quoted: Option<Py<EvMessage>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first.")
        })?;
        let jid = to.into_whatsapp_jid()?;
        let locals = get_current_locals(py)?;
        let context_info = Self::quote_context(py, quoted.as_ref());

        future_into_py_with_locals::<_, Py<SendResult>>(py, locals, async move {
            let mime = mimetype.unwrap_or_else(|| {
                guess_mimetype(&document_data).unwrap_or_else(|| "application/octet-stream".to_string())
            });
            let upload = client
                .upload(document_data, wacore::download::MediaType::Document, UploadOptions::default())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let mut doc = wa::DocumentMessage::default();
            doc.url = Some(upload.url);
            doc.mimetype = Some(mime);
            doc.title = file_name.clone();
            doc.file_sha256 = Some(upload.file_sha256.to_vec());
            doc.file_length = Some(upload.file_length);
            doc.media_key = Some(upload.media_key.to_vec());
            doc.file_name = file_name;
            doc.file_enc_sha256 = Some(upload.file_enc_sha256.to_vec());
            doc.direct_path = Some(upload.direct_path);
            doc.caption = caption;
            doc.context_info = context_info.into();
            let mut message = WhatsappMessage::default();
            message.document_message = buffa::MessageField::some(doc);

            let send_result = client
                .send_message(jid, message)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| Self::into_send_result(py, send_result))
        })
    }

    #[pyo3(signature = (to, audio_data, mimetype=None, ptt=false, seconds=None, quoted=None))]
    fn send_audio<'py>(
        &self,
        py: Python<'py>,
        to: PyJidArg<'py>,
        audio_data: Vec<u8>,
        mimetype: Option<String>,
        ptt: bool,
        seconds: Option<u32>,
        quoted: Option<Py<EvMessage>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first.")
        })?;
        let jid = to.into_whatsapp_jid()?;
        let locals = get_current_locals(py)?;
        let context_info = Self::quote_context(py, quoted.as_ref());

        future_into_py_with_locals::<_, Py<SendResult>>(py, locals, async move {
            let mime = mimetype.unwrap_or_else(|| {
                guess_mimetype(&audio_data).unwrap_or_else(|| "audio/ogg; codecs=opus".to_string())
            });
            let upload = client
                .upload(audio_data, wacore::download::MediaType::Audio, UploadOptions::default())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let mut audio = wa::AudioMessage::default();
            audio.url = Some(upload.url);
            audio.mimetype = Some(mime);
            audio.file_sha256 = Some(upload.file_sha256.to_vec());
            audio.file_length = Some(upload.file_length);
            audio.seconds = seconds;
            audio.ptt = Some(ptt);
            audio.media_key = Some(upload.media_key.to_vec());
            audio.file_enc_sha256 = Some(upload.file_enc_sha256.to_vec());
            audio.direct_path = Some(upload.direct_path);
            audio.context_info = context_info.into();
            let mut message = WhatsappMessage::default();
            message.audio_message = buffa::MessageField::some(audio);

            let send_result = client
                .send_message(jid, message)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| Self::into_send_result(py, send_result))
        })
    }

    #[pyo3(signature = (to, video_data, mimetype=None, caption=None, seconds=None, gif_playback=false, quoted=None))]
    fn send_video<'py>(
        &self,
        py: Python<'py>,
        to: PyJidArg<'py>,
        video_data: Vec<u8>,
        mimetype: Option<String>,
        caption: Option<String>,
        seconds: Option<u32>,
        gif_playback: bool,
        quoted: Option<Py<EvMessage>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first.")
        })?;
        let jid = to.into_whatsapp_jid()?;
        let locals = get_current_locals(py)?;
        let context_info = Self::quote_context(py, quoted.as_ref());

        future_into_py_with_locals::<_, Py<SendResult>>(py, locals, async move {
            let mime = mimetype.unwrap_or_else(|| {
                guess_mimetype(&video_data).unwrap_or_else(|| "video/mp4".to_string())
            });
            let upload = client
                .upload(video_data, wacore::download::MediaType::Video, UploadOptions::default())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let mut video = wa::VideoMessage::default();
            video.url = Some(upload.url);
            video.mimetype = Some(mime);
            video.file_sha256 = Some(upload.file_sha256.to_vec());
            video.file_length = Some(upload.file_length);
            video.seconds = seconds;
            video.media_key = Some(upload.media_key.to_vec());
            video.caption = caption;
            video.gif_playback = Some(gif_playback);
            video.file_enc_sha256 = Some(upload.file_enc_sha256.to_vec());
            video.direct_path = Some(upload.direct_path);
            video.context_info = context_info.into();
            let mut message = WhatsappMessage::default();
            message.video_message = buffa::MessageField::some(video);

            let send_result = client
                .send_message(jid, message)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| Self::into_send_result(py, send_result))
        })
    }

    #[pyo3(signature = (to, gif_data, caption=None, seconds=None, quoted=None))]
    fn send_gif<'py>(
        &self,
        py: Python<'py>,
        to: PyJidArg<'py>,
        gif_data: Vec<u8>,
        caption: Option<String>,
        seconds: Option<u32>,
        quoted: Option<Py<EvMessage>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.send_video(py, to, gif_data, Some("video/mp4".to_string()), caption, seconds, true, quoted)
    }

    #[pyo3(signature = (to, sticker_data, is_animated=false, quoted=None))]
    fn send_sticker<'py>(
        &self,
        py: Python<'py>,
        to: PyJidArg<'py>,
        sticker_data: Vec<u8>,
        is_animated: bool,
        quoted: Option<Py<EvMessage>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first.")
        })?;
        let jid = to.into_whatsapp_jid()?;
        let locals = get_current_locals(py)?;
        let context_info = Self::quote_context(py, quoted.as_ref());

        future_into_py_with_locals::<_, Py<SendResult>>(py, locals, async move {
            let mime = guess_mimetype(&sticker_data).unwrap_or_else(|| "image/webp".to_string());
            let upload = client
                .upload(sticker_data, wacore::download::MediaType::Sticker, UploadOptions::default())
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            let mut sticker = wa::StickerMessage::default();
            sticker.url = Some(upload.url);
            sticker.file_sha256 = Some(upload.file_sha256.to_vec());
            sticker.file_enc_sha256 = Some(upload.file_enc_sha256.to_vec());
            sticker.media_key = Some(upload.media_key.to_vec());
            sticker.mimetype = Some(mime);
            sticker.direct_path = Some(upload.direct_path);
            sticker.file_length = Some(upload.file_length);
            sticker.is_animated = Some(is_animated);
            sticker.context_info = context_info.into();
            let mut message = WhatsappMessage::default();
            message.sticker_message = buffa::MessageField::some(sticker);

            let send_result = client
                .send_message(jid, message)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| Self::into_send_result(py, send_result))
        })
    }

    #[pyo3(signature = (message_id, chat_jid, media_key, is_from_me=false, participant=None))]
    fn request_media_reupload<'py>(
        &self,
        py: Python<'py>,
        message_id: String,
        chat_jid: PyJidArg<'py>,
        media_key: &[u8],
        is_from_me: bool,
        participant: Option<PyJidArg<'py>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let media_key_arr: [u8; 32] = media_key.try_into().map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("media_key must be 32 bytes")
        })?;
        let msg_id = whatsapp_rust::MessageId::new(&message_id)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client is not running. Call Tryx.run() or Tryx.run_blocking() first.")
        })?;
        let chat_jid_value = chat_jid.into_whatsapp_jid()?;
        let participant_value = participant
            .map(|value| value.into_whatsapp_jid())
            .transpose()?;
        let locals = get_current_locals(py)?;

        future_into_py_with_locals::<_, MediaReuploadResult>(py, locals, async move {
            let target = whatsapp_rust::MessageRef::new(
                &chat_jid_value,
                msg_id,
                participant_value.as_ref(),
                is_from_me,
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            let req = whatsapp_rust::MediaReuploadRequest {
                target,
                media_key: &media_key_arr,
            };

            client
                .media_reupload()
                .request(&req)
                .await
                .map(MediaReuploadResult::from)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }
}
