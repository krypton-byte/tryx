use pyo3::pyclass;
use pyo3::prelude::*;
use crate::types::JID;

#[pyclass]
pub struct IsOnWhatsAppResult {
    #[pyo3(get)]
    jid: Py<JID>,
    #[pyo3(get)]
    is_registered: bool,
}

impl IsOnWhatsAppResult {
    pub fn new(jid: JID, is_registered: bool) -> Self {
        Python::attach(|py| {
            Self {
                jid: Py::new(py, JID::from(jid)).unwrap(),
                is_registered,
            }
        })
    }
}
impl From<wacore::iq::usync::IsOnWhatsAppResult> for IsOnWhatsAppResult {
    fn from(result: wacore::iq::usync::IsOnWhatsAppResult) -> Self {
        IsOnWhatsAppResult::new(result.jid.into(), result.is_registered)
    }
}



#[pyclass]
pub struct ContactInfo {
    #[pyo3(get)]
    jid: Py<JID>,
    #[pyo3(get)]
    lid: Option<Py<JID>>,
    #[pyo3(get)]
    is_registered: bool,
    #[pyo3(get)]
    is_business: bool,
    #[pyo3(get)]
    status: Option<String>,
    #[pyo3(get)]
    picture_id: Option<u64>,
}

impl ContactInfo {
    pub fn new(
        jid: JID,
        lid: Option<JID>,
        is_registered: bool,
        is_business: bool,
        status: Option<String>,
        picture_id: Option<u64>,
    ) -> Self {
        Python::attach(|py| {
            Self {
                jid: Py::new(py, JID::from(jid)).unwrap(),
                lid: lid.map(|l| Py::new(py, JID::from(l)).unwrap()),
                is_registered,
                is_business,
                status,
                picture_id,
            }
        })
    }
}


impl From<wacore::iq::usync::ContactInfo> for ContactInfo {
    fn from(info: wacore::iq::usync::ContactInfo) -> Self {
        ContactInfo::new(
            info.jid.into(),
            info.lid.map(Into::into),
            info.is_registered,
            info.is_business,
            info.status,
            info.picture_id,
        )
    }
}

#[pyclass]
pub struct UserInfo {
    #[pyo3(get)]
    jid: Py<JID>,
    #[pyo3(get)]
    lid: Option<Py<JID>>,
    #[pyo3(get)]
    status: Option<String>,
    #[pyo3(get)]
    picture_id: Option<String>,
    #[pyo3(get)]
    is_business: bool,
}

impl UserInfo {
    pub fn new(
        jid: JID,
        lid: Option<JID>,
        status: Option<String>,
        picture_id: Option<String>,
        is_business: bool,
    ) -> Self {
        Python::attach(|py| {
            Self {
                jid: Py::new(py, JID::from(jid)).unwrap(),
                lid: lid.map(|l| Py::new(py, JID::from(l)).unwrap()),
                status,
                picture_id,
                is_business,
            }
        })
    }
}

impl From<wacore::iq::usync::UserInfo> for UserInfo {
    fn from(info: wacore::iq::usync::UserInfo) -> Self {
        UserInfo::new(
            info.jid.into(),
            info.lid.map(Into::into),
            info.status,
            info.picture_id,
            info.is_business,
        )
    }
}

