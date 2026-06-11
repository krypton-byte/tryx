use pyo3::prelude::*;
use pyo3::types::PyBytes;
use wacore::store::Device;
use wacore::store::traits::{LidPnMappingEntry, DeviceListRecord, MsgSecretEntry, AppStateSyncKey, TcTokenEntry};
use wacore_appstate::processor::AppStateMutationMAC;
use wacore::appstate::hash::HashState;

macro_rules! define_py_wrapper {
    ($py_name:ident, $rs_name:ty, $name_str:expr) => {
        #[pyclass(name = $name_str, module="tryx.backend")]
        #[derive(Clone)]
        pub struct $py_name(pub $rs_name);

        #[pymethods]
        impl $py_name {
            pub fn to_bytes<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyBytes>> {
                let bytes = serde_json::to_vec(&self.0).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                Ok(PyBytes::new(py, &bytes))
            }
            
            #[staticmethod]
            pub fn from_bytes(bytes: &[u8]) -> PyResult<Self> {
                let inner = serde_json::from_slice(bytes).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                Ok(Self(inner))
            }

            pub fn __getstate__<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyBytes>> {
                let bytes = serde_json::to_vec(&self.0).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                Ok(PyBytes::new(py, &bytes))
            }

            pub fn __setstate__(&mut self, state: &Bound<PyBytes>) -> PyResult<()> {
                self.0 = serde_json::from_slice(state.as_bytes()).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                Ok(())
            }
        }
    };
}

define_py_wrapper!(PyDevice, Device, "Device");
define_py_wrapper!(PyLidPnMappingEntry, LidPnMappingEntry, "LidPnMappingEntry");
define_py_wrapper!(PyDeviceListRecord, DeviceListRecord, "DeviceListRecord");
define_py_wrapper!(PyTcTokenEntry, TcTokenEntry, "TcTokenEntry");
define_py_wrapper!(PyAppStateSyncKey, AppStateSyncKey, "AppStateSyncKey");
define_py_wrapper!(PyHashState, HashState, "HashState");
define_py_wrapper!(PyAppStateMutationMAC, AppStateMutationMAC, "AppStateMutationMAC");
define_py_wrapper!(PyMsgSecretEntry, MsgSecretEntry, "MsgSecretEntry");
