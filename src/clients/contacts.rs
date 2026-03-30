use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_async_runtimes::tokio::{future_into_py_with_locals, get_current_locals};
use tokio::sync::watch;
use whatsapp_rust::{Client, Jid as WaJid};
use wacore::iq::usync::{
    IsOnWhatsAppQueryType,
    IsOnWhatsAppSpec,
    IsOnWhatsAppUser,
    UserInfoSpec,
};

use crate::types::{JID, ProfilePicture};
use crate::wacore::iq::usync::{ContactInfo, IsOnWhatsAppResult, UserInfo};

#[pyclass]
pub struct ContactClient {
    pub client_rx: watch::Receiver<Option<Arc<Client>>>,
}

impl ContactClient {
    fn get_client(&self) -> PyResult<Arc<Client>> {
        self.client_rx
            .borrow()
            .clone()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Bot is not running"))
    }
}

#[pymethods]
impl ContactClient {
    fn get_info<'py>(&self, py: Python<'py>, phones: Vec<String>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;

        future_into_py_with_locals::<_, Vec<Py<ContactInfo>>>(py, locals, async move {
            let users = phones
                .iter()
                .map(|phone| IsOnWhatsAppUser {
                    jid: WaJid::pn(phone).to_non_ad(),
                    known_lid: None,
                })
                .collect::<Vec<_>>();
            let spec = IsOnWhatsAppSpec::new(
                users,
                wacore::time::now_millis().to_string(),
                IsOnWhatsAppQueryType::Pn,
            );
            let info = client
                .execute(spec)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            let result = Python::attach(|py| {
                info.into_iter()
                    .map(|item| Py::new(py, ContactInfo::from(item)))
                    .collect::<PyResult<Vec<_>>>()
            })?;

            Ok(result)
        })
    }

    fn get_user_info<'py>(&self, py: Python<'py>, jid: Py<JID>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let locals = get_current_locals(py)?;
        let jid_value = jid.bind(py).borrow().as_whatsapp_jid();
        let spec = UserInfoSpec::new(vec![jid_value], wacore::time::now_millis().to_string());

        future_into_py_with_locals::<_, Py<PyDict>>(py, locals, async move {
            let info = client
                .execute(spec)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            Python::attach(|py| {
                let dict = PyDict::new(py);
                for (jid, info) in info {
                    let contact_jid = jid.clone();
                    let contact_info = UserInfo::new(
                        contact_jid.into(),
                        info.lid.as_ref().map(|l| JID::from(l.clone())),
                        info.is_business,
                        info.status,
                        info.picture_id,
                    );
                    dict.set_item(JID::from(jid), contact_info)?;
                }
                Ok(dict.unbind())
            })
        })
    }

    fn get_profile_picture<'py>(
        &self,
        py: Python<'py>,
        jid: Py<JID>,
        preview: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;
        let jid_obj = jid.bind(py).borrow().as_whatsapp_jid();
        let locals = get_current_locals(py)?;

        future_into_py_with_locals(py, locals, async move {
            let pic = client
                .contacts()
                .get_profile_picture(&jid_obj, preview)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
                .ok_or(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Profile picture not found",
                ))?;
            Ok(ProfilePicture::from(pic))
        })
    }

    fn is_on_whatsapp<'py>(&self, py: Python<'py>, jid: Vec<Py<JID>>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.get_client()?;

        let jid_values = jid
            .into_iter()
            .map(|item| {
                let s_jid = item.borrow(py);
                s_jid.as_whatsapp_jid()
            })
            .collect::<Vec<_>>();

        let locals = get_current_locals(py)?;
        future_into_py_with_locals::<_, Vec<IsOnWhatsAppResult>>(py, locals, async move {
            let mut pn_users = Vec::new();
            let mut lid_users = Vec::new();

            for jid in jid_values {
                if jid.is_pn() {
                    pn_users.push(IsOnWhatsAppUser {
                        jid: jid.to_non_ad(),
                        known_lid: None,
                    });
                } else if jid.is_lid() {
                    lid_users.push(IsOnWhatsAppUser {
                        jid: jid.to_non_ad(),
                        known_lid: None,
                    });
                }
            }

            let mut response = Vec::new();
            if !pn_users.is_empty() {
                let spec = IsOnWhatsAppSpec::new(
                    pn_users,
                    wacore::time::now_millis().to_string(),
                    IsOnWhatsAppQueryType::Pn,
                );
                response.extend(
                    client
                        .execute(spec)
                        .await
                        .map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                        })?,
                );
            }

            if !lid_users.is_empty() {
                let spec = IsOnWhatsAppSpec::new(
                    lid_users,
                    wacore::time::now_millis().to_string(),
                    IsOnWhatsAppQueryType::Lid,
                );
                response.extend(
                    client
                        .execute(spec)
                        .await
                        .map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                        })?,
                );
            }

            let response_py = response
                .into_iter()
                .map(IsOnWhatsAppResult::from)
                .collect::<Vec<_>>();
            Ok(response_py)
        })
    }
}