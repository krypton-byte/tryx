use pyo3::prelude::*;
use whatsapp_rust::PresenceStatus as WaPresenceStatus;

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresenceStatus {
    Available = 0,
    Unavailable = 1,
}

impl From<WaPresenceStatus> for PresenceStatus {
    fn from(value: WaPresenceStatus) -> Self {
        match value {
            WaPresenceStatus::Available => Self::Available,
            WaPresenceStatus::Unavailable => Self::Unavailable,
            _ => Self::Unavailable, // Default case for any unknown status
        }
    }
}

impl From<PresenceStatus> for WaPresenceStatus {
    fn from(value: PresenceStatus) -> Self {
        match value {
            PresenceStatus::Available => Self::Available,
            PresenceStatus::Unavailable => Self::Unavailable,
        }
    }
}
