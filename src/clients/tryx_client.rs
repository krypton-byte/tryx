use std::sync::{Arc};
use pyo3::{Bound, PyAny, pyclass, pymethods};
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use waproto::whatsapp::Message as WhatsappMessage;
use waproto::whatsapp::message::{self as wa};
use wacore::proto_helpers::build_quote_context;
use prost::Message;
use whatsapp_rust::Client;
use crate::clients::chat_actions::ChatActionsClient;
use crate::clients::community::CommunityClient;
use crate::clients::contacts::ContactClient;
use crate::clients::groups::GroupsClient;
use crate::clients::newsletter::NewsletterClient;
use crate::clients::status::StatusClient;
use crate::events::types::{EvMessage};
use crate::types::{JID, UploadResponse};
use crate::wacore::download::MediaType;
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
}

#[pymethods]
impl TryxClient {
    fn is_connected(&self) -> bool {
        self.client_rx.borrow().is_some()
    }
    // fn download_media_to_writter<'py>(&self, py: Python<'py>, message: Py<PyAny>, path: String) -> PyResult<Bound<'py, PyAny>> {
    //     let client = self.client_rx.borrow().clone().ok_or_else(|| {
    //         PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running")
    //     })?;
    //     let message_type_name = message
    //         .getattr(py, "DESCRIPTOR")
    //         .and_then(|descriptor| descriptor.getattr(py, "name"))
    //         .and_then(|name| name.extract::<String>(py))
    //         .unwrap_or_default();
    //     let serialized: Vec<u8> = message
    //         .call_method0(py, "SerializeToString")?
    //         .extract(py)?;

    //     let locals = get_current_locals(py)?;
    //     future_into_py_with_locals(py, locals, async move {
    //         match message_type_name.as_str() {
    //             "ImageMessage" => {
    //                 let media = wa::ImageMessage::decode(serialized.as_slice()).map_err(|e| {
    //                     PyErr::new::<pyo3::exceptions::PyValueError, _>(
    //                         format!("Failed to decode ImageMessage: {}", e),
    //                     )
    //                 })?;
    //                 client.download_to_writer(&media, path).await
    //             }
    //             "VideoMessage" => {
    //                 let media = wa::VideoMessage::decode(serialized.as_slice()).map_err(|e| {
    //                     PyErr::new::<pyo3::exceptions::PyValueError, _>(
    //                         format!("Failed to decode VideoMessage: {}", e),
    //                     )
    //                 })?;
    //                 client.download_to_writer(&media, path).await
    //             }
    //             "DocumentMessage" => {
    //                 let media = wa::DocumentMessage::decode(serialized.as_slice()).map_err(|e| {
    //                     PyErr::new::<pyo3::exceptions::PyValueError, _>(
    //                         format!("Failed to decode DocumentMessage: {}", e),
    //                     )
    //                 })?;
    //                 client.download_to_writer(&media, path).await
    //             }
    //             "AudioMessage" => {
    //                 let media = wa::AudioMessage::decode(serialized.as_slice()).map_err(|e| {
    //                     PyErr::new::<pyo3::exceptions::PyValueError, _>(
    //                         format!("Failed to decode AudioMessage: {}", e),
    //                     )
    //                 })?;
    //                 client.download_to_writer(&media, path).await
    //             }
    //             "StickerMessage" => {
    //                 let media = wa::StickerMessage::decode(serialized.as_slice()).map_err(|e| {
    //                     PyErr::new::<pyo3::exceptions::PyValueError, _>(
    //                         format!("Failed to decode StickerMessage: {}", e),
    //                     )
    //                 })?;
    //                 client.download_to_writer(&media, path).await
    //             }
    //             _ => {
    //                 // Fallback path for unknown wrappers from Python side.
    //                 if let Ok(media) = wa::ImageMessage::decode(serialized.as_slice()) {
    //                     client.download_to_writer(&media, path).await
    //                 } else if let Ok(media) = wa::VideoMessage::decode(serialized.as_slice()) {
    //                     client.download_to_writer(&media, path).await
    //                 } else if let Ok(media) = wa::DocumentMessage::decode(serialized.as_slice()) {
    //                     client.download_to_writer(&media, path).await
    //                 } else if let Ok(media) = wa::AudioMessage::decode(serialized.as_slice()) {
    //                     client.download_to_writer(&media, path).await
    //                 } else if let Ok(media) = wa::StickerMessage::decode(serialized.as_slice()) {
    //                     client.download_to_writer(&media, path).await
    //                 } else {
    //                     return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
    //                         "Failed to decode message as supported media message",
    //                     ));
    //                 }
    //             }
    //         }.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
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
        future_into_py_with_locals::<_, Vec<u8>>(py, locals, async move {
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
    #[pyo3(signature = (to, text, quoted=None))]
    fn send_text<'py>(&self, py: Python<'py>, to: Py<JID>, text: String, quoted: Option<Py<EvMessage>>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running")
        })?;
        let jid = to.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;
        let context_info = quoted.as_ref().map(|q| {
            let quote = q.bind(py).borrow();
            let msg = quote.inner.as_ref();
            build_quote_context(
                quote.inner_message_info.id.clone(),
                quote.inner_message_info.source.chat.clone(),
                msg,
            )
        });
        future_into_py_with_locals(py, locals, async move {
            match quoted {
                Some(_) => {
                    let message = WhatsappMessage {
                        extended_text_message: Some(Box::new(wa::ExtendedTextMessage {
                            text: Some(text),
                            context_info: context_info.map(Box::new),
                            ..Default::default()
                        })),
                        ..Default::default()
                    };
                    let message_id = client
                        .send_message(jid, message)
                        .await
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
                    Ok(message_id.to_string())
                }
                None => {
                    let message = WhatsappMessage {
                        conversation: Some(text),
                        ..Default::default()
                    };
                    let message_id = client
                        .send_message(jid, message)
                        .await
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
                    Ok(message_id.to_string())
                }
            }
        })
    }
    #[pyo3(signature = (to, photo_data, caption, quoted=None))]
    fn send_photo<'py>(&self, py: Python<'py>, to: Py<JID>, photo_data: &[u8], caption: String, quoted: Option<Py<EvMessage>>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client_rx.borrow().clone().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running")
        })?;
        let jid = to.bind(py).borrow().as_whatsapp_jid();
        let photo_clone = photo_data.to_vec();
        let locals = get_current_locals(py)?;
        let context_info = quoted.as_ref().map(|q| {
            let quote = q.bind(py).borrow();
            let msg = quote.inner.as_ref();
            build_quote_context(
                quote.inner_message_info.id.clone(),
                quote.inner_message_info.source.chat.clone(),
                msg,
            )
        });
        future_into_py_with_locals(py, locals, async move {
            let upload = client
                .upload(photo_clone, wacore::download::MediaType::Image)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            // let image_message = wa::ImageMessage {
            //     url: Some(upload.url),
            //     direct_path: Some(upload.direct_path),
            //     media_key: Some(upload.media_key),
            //     file_enc_sha256: Some(upload.file_enc_sha256),
            //     file_sha256: Some(upload.file_sha256),
            //     file_length: Some(upload.file_length),
            //     caption: Some(caption),
            //     ..Default::default()
            // };
            let message = WhatsappMessage {
                image_message: Some(Box::new(wa::ImageMessage {
                url: Some(upload.url),
                direct_path: Some(upload.direct_path),
                media_key: Some(upload.media_key),
                file_enc_sha256: Some(upload.file_enc_sha256),
                file_sha256: Some(upload.file_sha256),
                file_length: Some(upload.file_length),
                caption: Some(caption),
                context_info: context_info.map(Box::new),
                ..Default::default()
            })),
                ..Default::default()
            };
            let message_id = client
                .send_message(jid, message)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(message_id.to_string())
        })
    }
}

