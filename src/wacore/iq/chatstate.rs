use pyo3::prelude::*;
use whatsapp_rust::ChatStateType as WaChatStateType;

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatStateType {
    Composing = 0,
    Recording = 1,
    Paused = 2,
}

impl From<WaChatStateType> for ChatStateType {
    fn from(value: WaChatStateType) -> Self {
        match value {
            WaChatStateType::Composing => Self::Composing,
            WaChatStateType::Recording => Self::Recording,
            WaChatStateType::Paused => Self::Paused,
        }
    }
}

impl From<ChatStateType> for WaChatStateType {
    fn from(value: ChatStateType) -> Self {
        match value {
            ChatStateType::Composing => Self::Composing,
            ChatStateType::Recording => Self::Recording,
            ChatStateType::Paused => Self::Paused,
        }
    }
}