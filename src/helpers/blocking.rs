use pyo3::{Py, pyclass, pymethods};

use crate::types::JID;

#[pyclass]
pub struct BlockingHelpers;

#[pymethods]
impl BlockingHelpers {
    #[staticmethod]
    fn same_user(a: Py<JID>, b: Py<JID>, py: pyo3::Python<'_>) -> bool {
        let a_jid = a.bind(py).borrow().as_whatsapp_jid();
        let b_jid = b.bind(py).borrow().as_whatsapp_jid();
        a_jid.user == b_jid.user
    }
}
