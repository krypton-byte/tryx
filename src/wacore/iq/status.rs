use pyo3::prelude::*;
use whatsapp_rust::{
    StatusPrivacySetting as WaStatusPrivacySetting,
    StatusSendOptions as WaStatusSendOptions,
};

#[pyclass(eq, eq_int, from_py_object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusPrivacySetting {
    Contacts = 0,
    AllowList = 1,
    DenyList = 2,
}

impl From<WaStatusPrivacySetting> for StatusPrivacySetting {
    fn from(value: WaStatusPrivacySetting) -> Self {
        match value {
            WaStatusPrivacySetting::Contacts => Self::Contacts,
            WaStatusPrivacySetting::AllowList => Self::AllowList,
            WaStatusPrivacySetting::DenyList => Self::DenyList,
            _ => Self::Contacts,
        }
    }
}

impl From<StatusPrivacySetting> for WaStatusPrivacySetting {
    fn from(value: StatusPrivacySetting) -> Self {
        match value {
            StatusPrivacySetting::Contacts => Self::Contacts,
            StatusPrivacySetting::AllowList => Self::AllowList,
            StatusPrivacySetting::DenyList => Self::DenyList,
        }
    }
}

#[pyclass]
pub struct StatusSendOptions {
    #[pyo3(get, set)]
    pub privacy: StatusPrivacySetting,
}

#[pymethods]
impl StatusSendOptions {
    #[new]
    #[pyo3(signature = (privacy=StatusPrivacySetting::Contacts))]
    fn new(privacy: StatusPrivacySetting) -> Self {
        Self { privacy }
    }
}

impl StatusSendOptions {
    pub fn to_rust_options(&self) -> WaStatusSendOptions {
        WaStatusSendOptions {
            privacy: self.privacy.into(),
        }
    }
}
