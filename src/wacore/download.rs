use pyo3::pyclass;
use wacore::download::{MediaType as WACoreMediaType};

#[pyclass]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Document,
    History,
    AppState,
    Sticker,
    StickerPack,
    LinkThumbnail,
}
impl From<MediaType> for WACoreMediaType {
    fn from(value: MediaType) -> Self {
        match value {
            MediaType::Image => WACoreMediaType::Image,
            MediaType::Video => WACoreMediaType::Video,
            MediaType::Audio => WACoreMediaType::Audio,
            MediaType::Document => WACoreMediaType::Document,
            MediaType::History => WACoreMediaType::History,
            MediaType::AppState => WACoreMediaType::AppState,
            MediaType::Sticker => WACoreMediaType::Sticker,
            MediaType::StickerPack => WACoreMediaType::StickerPack,
            MediaType::LinkThumbnail => WACoreMediaType::LinkThumbnail,
        }
    }
}

impl MediaType {
    pub fn to_wacore_enum(&self) -> WACoreMediaType {
        match &self {
            MediaType::Image => WACoreMediaType::Image,
            MediaType::Video => WACoreMediaType::Video,
            MediaType::Audio => WACoreMediaType::Audio,
            MediaType::Document => WACoreMediaType::Document,
            MediaType::History => WACoreMediaType::History,
            MediaType::AppState => WACoreMediaType::AppState,
            MediaType::Sticker => WACoreMediaType::Sticker,
            MediaType::StickerPack => WACoreMediaType::StickerPack,
            MediaType::LinkThumbnail => WACoreMediaType::LinkThumbnail,
        }
    }
}