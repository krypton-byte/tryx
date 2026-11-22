use pyo3::prelude::*;
use whatsapp_rust::ChatActivity as WaChatActivity;

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatStateType {
    Composing = 0,
    Recording = 1,
    Paused = 2,
}

impl From<WaChatActivity> for ChatStateType {
    fn from(value: WaChatActivity) -> Self {
        match value {
            WaChatActivity::Typing => Self::Composing,
            WaChatActivity::RecordingAudio => Self::Recording,
            WaChatActivity::Idle => Self::Paused,
            _ => Self::Paused,
        }
    }
}

impl From<ChatStateType> for WaChatActivity {
    fn from(value: ChatStateType) -> Self {
        match value {
            ChatStateType::Composing => Self::Typing,
            ChatStateType::Recording => Self::RecordingAudio,
            ChatStateType::Paused => Self::Idle,
        }
    }
}