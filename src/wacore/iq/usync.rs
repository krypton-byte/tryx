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
        IsOnWhatsAppResult {
            jid: Python::attach(|py| Py::new(py, JID::from(result.jid)).unwrap()),
            is_registered: result.is_registered,
        }
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
        Python::attach(|py|{
            ContactInfo {
            jid: Py::new(py, JID::from(info.jid)).unwrap(),
            lid: info.lid.map(|l| Py::new(py, JID::from(l)).unwrap()),
            is_registered: info.is_registered,
            is_business: info.is_business,
            status: info.status,
            picture_id: info.picture_id,
        }
        })
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
        Python::attach(|py| {
            UserInfo {
                jid: Py::new(py, JID::from(info.jid)).unwrap(),
                lid: info.lid.map(|l| Py::new(py, JID::from(l)).unwrap()),
                status: info.status,
                picture_id: info.picture_id,
                is_business: info.is_business,
            }
        })
    }
}

