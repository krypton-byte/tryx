import re

def process_method(m):
    name, args, ret = m
    args_list = []
    py_args = []
    if args.strip():
        for arg in args.split(','):
            arg = arg.strip()
            arg_name, arg_type = arg.split(':')
            arg_name = arg_name.strip()
            arg_type = arg_type.strip()
            args_list.append(arg)
            
            if arg_type == "&wacore::store::Device":
                py_args.append(f"crate::backend::store_types::PyDevice({arg_name}.clone())")
            elif arg_type == "&LidPnMappingEntry":
                py_args.append(f"crate::backend::store_types::PyLidPnMappingEntry({arg_name}.clone())")
            elif arg_type == "DeviceListRecord":
                py_args.append(f"crate::backend::store_types::PyDeviceListRecord({arg_name}.clone())")
            elif arg_type == "&TcTokenEntry":
                py_args.append(f"crate::backend::store_types::PyTcTokenEntry({arg_name}.clone())")
            elif arg_type == "AppStateSyncKey":
                py_args.append(f"crate::backend::store_types::PyAppStateSyncKey({arg_name}.clone())")
            elif arg_type == "HashState":
                py_args.append(f"crate::backend::store_types::PyHashState({arg_name}.clone())")
            elif arg_type == "&[AppStateMutationMAC]":
                # Convert slice of AppStateMutationMAC to Vec of PyAppStateMutationMAC
                py_args.append(f"{arg_name}.iter().map(|x| crate::backend::store_types::PyAppStateMutationMAC(x.clone())).collect::<Vec<_>>()")
            elif arg_type == "Vec<MsgSecretEntry>":
                # Convert Vec of MsgSecretEntry to Vec of PyMsgSecretEntry
                py_args.append(f"{arg_name}.into_iter().map(|x| crate::backend::store_types::PyMsgSecretEntry(x)).collect::<Vec<_>>()")
            elif arg_type in ("&str",):
                py_args.append(f"{arg_name}")
            elif arg_type in ("&[u8]",):
                py_args.append(f"pyo3::types::PyBytes::new(_py, {arg_name})")
            elif arg_type in ("&[&str]",):
                py_args.append(f"{arg_name}.to_vec()")
            elif arg_type in ("&[Vec<u8>]",):
                py_args.append(f"pyo3::types::PyList::new(_py, {arg_name}.iter().map(|x| pyo3::types::PyBytes::new(_py, x)))")
            elif arg_type in ("&[(&str, bool)]",):
                py_args.append(f"{arg_name}.to_vec()")
            elif arg_type in ("[u8; 32]", "i64", "u32", "u64", "bool"):
                if arg_type == "[u8; 32]":
                    py_args.append(f"pyo3::types::PyBytes::new(_py, &{arg_name})")
                else:
                    py_args.append(f"{arg_name}")
            else:
                py_args.append(f"{arg_name}")
    
    args_str = ", ".join(args_list)
    py_args_tuple = ", ".join(py_args)
    if len(py_args) == 1:
        py_args_tuple += ","
    
    # Return type extraction
    
    ret_extraction = ""
    if ret == "StoreResult<()>":
        ret_extraction = "Ok(())"
    elif ret == "StoreResult<bool>":
        ret_extraction = "_res.bind(_py).extract::<bool>().map_err(|e| make_err(e.to_string()))"
    elif ret == "StoreResult<i32>":
        ret_extraction = "_res.bind(_py).extract::<i32>().map_err(|e| make_err(e.to_string()))"
    elif ret == "StoreResult<u32>":
        ret_extraction = "_res.bind(_py).extract::<u32>().map_err(|e| make_err(e.to_string()))"
    elif ret == "StoreResult<usize>":
        ret_extraction = "_res.bind(_py).extract::<usize>().map_err(|e| make_err(e.to_string()))"
    elif ret == "StoreResult<Option<wacore::store::Device>>":
        ret_extraction = "if _res.bind(_py).is_none() { Ok(None) } else { let v = _res.bind(_py).extract::<pyo3::PyRef<crate::backend::store_types::PyDevice>>().map_err(|e| make_err(e.to_string()))?; Ok(Some(v.0.clone())) }"
    elif ret == "StoreResult<Option<LidPnMappingEntry>>":
        ret_extraction = "if _res.bind(_py).is_none() { Ok(None) } else { let v = _res.bind(_py).extract::<pyo3::PyRef<crate::backend::store_types::PyLidPnMappingEntry>>().map_err(|e| make_err(e.to_string()))?; Ok(Some(v.0.clone())) }"
    elif ret == "StoreResult<Vec<LidPnMappingEntry>>":
        ret_extraction = "let v = _res.bind(_py).extract::<Vec<pyo3::PyRef<crate::backend::store_types::PyLidPnMappingEntry>>>().map_err(|e| make_err(e.to_string()))?; Ok(v.into_iter().map(|x| x.0.clone()).collect())"
    elif ret == "StoreResult<Option<DeviceListRecord>>":
        ret_extraction = "if _res.bind(_py).is_none() { Ok(None) } else { let v = _res.bind(_py).extract::<pyo3::PyRef<crate::backend::store_types::PyDeviceListRecord>>().map_err(|e| make_err(e.to_string()))?; Ok(Some(v.0.clone())) }"
    elif ret == "StoreResult<Option<TcTokenEntry>>":
        ret_extraction = "if _res.bind(_py).is_none() { Ok(None) } else { let v = _res.bind(_py).extract::<pyo3::PyRef<crate::backend::store_types::PyTcTokenEntry>>().map_err(|e| make_err(e.to_string()))?; Ok(Some(v.0.clone())) }"
    elif ret == "StoreResult<Option<AppStateSyncKey>>":
        ret_extraction = "if _res.bind(_py).is_none() { Ok(None) } else { let v = _res.bind(_py).extract::<pyo3::PyRef<crate::backend::store_types::PyAppStateSyncKey>>().map_err(|e| make_err(e.to_string()))?; Ok(Some(v.0.clone())) }"
    elif ret == "StoreResult<HashState>":
        ret_extraction = "let v = _res.bind(_py).extract::<pyo3::PyRef<crate::backend::store_types::PyHashState>>().map_err(|e| make_err(e.to_string()))?; Ok(v.0.clone())"
    elif ret == "StoreResult<Vec<(String, bool)>>":
        ret_extraction = "let v = _res.bind(_py).extract::<Vec<(String, bool)>>().map_err(|e| make_err(e.to_string()))?; Ok(v)"
    elif ret == "StoreResult<Vec<String>>":
        ret_extraction = "let v = _res.bind(_py).extract::<Vec<String>>().map_err(|e| make_err(e.to_string()))?; Ok(v)"
    elif ret == "StoreResult<Option<Vec<u8>>>":
        ret_extraction = "if _res.bind(_py).is_none() { Ok(None) } else { Ok(Some(_res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?)) }"
    elif ret == "StoreResult<Option<Bytes>>":
        ret_extraction = "if _res.bind(_py).is_none() { Ok(None) } else { Ok(Some(Bytes::from(_res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?))) }"
    elif ret == "StoreResult<Option<[u8; 32]>>":
        ret_extraction = "if _res.bind(_py).is_none() { Ok(None) } else { let v = _res.bind(_py).extract::<Vec<u8>>().map_err(|e| make_err(e.to_string()))?; let mut arr = [0u8; 32]; arr.copy_from_slice(&v); Ok(Some(arr)) }"
    elif ret == "StoreResult<Vec<(u32, Vec<u8>)>>":
        ret_extraction = "let v = _res.bind(_py).extract::<Vec<(u32, Vec<u8>)>>().map_err(|e| make_err(e.to_string()))?; Ok(v)"
    else:
        ret_extraction = "UNHANDLED_RET"

    call_method_str = f'let coro = self.py_obj.bind(_py).call_method1("{name}", ({py_args_tuple})).map_err(|e| make_err(e.to_string()))?;'
    if not py_args:
        call_method_str = f'let coro = self.py_obj.bind(_py).call_method0("{name}").map_err(|e| make_err(e.to_string()))?;'

    return f"""    async fn {name}(&self, {args_str}) -> {ret} {{
        let fut = Python::attach(|_py| {{
            {call_method_str}
            pyo3_async_runtimes::tokio::into_future(coro).map_err(|e| make_err(e.to_string()))
        }})?;
        let _res = fut.await.map_err(|e| make_err(e.to_string()))?;
        Python::attach(|_py| {{
            {ret_extraction}
        }})
    }}"""

methods = [
    # DeviceStore
    ("save", "device: &wacore::store::Device", "StoreResult<()>"),
    ("load", "", "StoreResult<Option<wacore::store::Device>>"),
    ("exists", "", "StoreResult<bool>"),
    ("create", "", "StoreResult<i32>"),
    # ProtocolStore
    ("get_sender_key_devices", "group_jid: &str", "StoreResult<Vec<(String, bool)>>"),
    ("set_sender_key_status", "group_jid: &str, entries: &[(&str, bool)]", "StoreResult<()>"),
    ("clear_sender_key_devices", "group_jid: &str", "StoreResult<()>"),
    ("delete_sender_key_device_rows", "device_jids: &[&str]", "StoreResult<()>"),
    ("clear_all_sender_key_devices", "", "StoreResult<()>"),
    ("get_lid_mapping", "lid: &str", "StoreResult<Option<LidPnMappingEntry>>"),
    ("get_pn_mapping", "phone: &str", "StoreResult<Option<LidPnMappingEntry>>"),
    ("put_lid_mapping", "entry: &LidPnMappingEntry", "StoreResult<()>"),
    ("get_all_lid_mappings", "", "StoreResult<Vec<LidPnMappingEntry>>"),
    ("save_base_key", "address: &str, message_id: &str, base_key: &[u8]", "StoreResult<()>"),
    ("has_same_base_key", "address: &str, message_id: &str, current_base_key: &[u8]", "StoreResult<bool>"),
    ("delete_base_key", "address: &str, message_id: &str", "StoreResult<()>"),
    ("update_device_list", "record: DeviceListRecord", "StoreResult<()>"),
    ("get_devices", "user: &str", "StoreResult<Option<DeviceListRecord>>"),
    ("delete_devices", "user: &str", "StoreResult<()>"),
    ("get_tc_token", "jid: &str", "StoreResult<Option<TcTokenEntry>>"),
    ("put_tc_token", "jid: &str, entry: &TcTokenEntry", "StoreResult<()>"),
    ("delete_tc_token", "jid: &str", "StoreResult<()>"),
    ("get_all_tc_token_jids", "", "StoreResult<Vec<String>>"),
    ("delete_expired_tc_tokens", "cutoff: i64", "StoreResult<u32>"),
    ("store_sent_message", "chat_jid: &str, message_id: &str, payload: &[u8]", "StoreResult<()>"),
    ("take_sent_message", "chat_jid: &str, message_id: &str", "StoreResult<Option<Vec<u8>>>"),
    ("delete_expired_sent_messages", "cutoff: i64", "StoreResult<u32>"),
    # AppSyncStore
    ("get_sync_key", "key_id: &[u8]", "StoreResult<Option<AppStateSyncKey>>"),
    ("set_sync_key", "key_id: &[u8], key: AppStateSyncKey", "StoreResult<()>"),
    ("get_version", "name: &str", "StoreResult<HashState>"),
    ("set_version", "name: &str, state: HashState", "StoreResult<()>"),
    ("put_mutation_macs", "name: &str, version: u64, mutations: &[AppStateMutationMAC]", "StoreResult<()>"),
    ("get_mutation_mac", "name: &str, index_mac: &[u8]", "StoreResult<Option<Vec<u8>>>"),
    ("delete_mutation_macs", "name: &str, index_macs: &[Vec<u8>]", "StoreResult<()>"),
    ("get_latest_sync_key_id", "", "StoreResult<Option<Vec<u8>>>"),
    # SignalStore
    ("put_identity", "address: &str, key: [u8; 32]", "StoreResult<()>"),
    ("load_identity", "address: &str", "StoreResult<Option<[u8; 32]>>"),
    ("delete_identity", "address: &str", "StoreResult<()>"),
    ("get_session", "address: &str", "StoreResult<Option<Bytes>>"),
    ("put_session", "address: &str, session: &[u8]", "StoreResult<()>"),
    ("delete_session", "address: &str", "StoreResult<()>"),
    ("store_prekey", "id: u32, record: &[u8], uploaded: bool", "StoreResult<()>"),
    ("load_prekey", "id: u32", "StoreResult<Option<Bytes>>"),
    ("remove_prekey", "id: u32", "StoreResult<()>"),
    ("get_max_prekey_id", "", "StoreResult<u32>"),
    ("store_signed_prekey", "id: u32, record: &[u8]", "StoreResult<()>"),
    ("load_signed_prekey", "id: u32", "StoreResult<Option<Vec<u8>>>"),
    ("load_all_signed_prekeys", "", "StoreResult<Vec<(u32, Vec<u8>)>>"),
    ("remove_signed_prekey", "id: u32", "StoreResult<()>"),
    ("put_sender_key", "address: &str, record: &[u8]", "StoreResult<()>"),
    ("get_sender_key", "address: &str", "StoreResult<Option<Vec<u8>>>"),
    ("delete_sender_key", "address: &str", "StoreResult<()>"),
    # MsgSecretStore
    ("put_msg_secrets", "entries: Vec<MsgSecretEntry>", "StoreResult<usize>"),
    ("get_msg_secret", "chat: &str, sender: &str, msg_id: &str", "StoreResult<Option<Vec<u8>>>"),
    ("delete_expired_msg_secrets", "cutoff: i64", "StoreResult<u32>"),
]

out = """
use async_trait::async_trait;
use bytes::Bytes;
use pyo3::prelude::*;
use wacore::appstate::hash::HashState;
use wacore::store::traits::*;
use wacore::store::error::{Result as StoreResult, StoreError};
use wacore_appstate::processor::AppStateMutationMAC;
use wacore::store::traits::TcTokenEntry;
use wacore::store::traits::{LidPnMappingEntry, DeviceListRecord, MsgSecretEntry, AppStateSyncKey};

fn make_err(msg: impl ToString) -> StoreError {
    StoreError::Io(std::io::Error::new(std::io::ErrorKind::Other, msg.to_string()))
}

pub struct PythonStore {
    pub py_obj: Py<crate::backend::BackendBase>,
}

impl Clone for PythonStore {
    fn clone(&self) -> Self {
        Python::attach(|_py| Self {
            py_obj: self.py_obj.clone_ref(_py)
        })
    }
}

impl PythonStore {
    pub fn new(py_obj: Py<crate::backend::BackendBase>) -> Self {
        Self { py_obj }
    }
}
"""

group_to_trait = {
    0: "DeviceStore",
    4: "ProtocolStore",
    27: "AppSyncStore",
    35: "SignalStore",
    51: "MsgSecretStore"
}

current_group = -1
for i, m in enumerate(methods):
    if i in group_to_trait:
        if i != 0:
            out += "}\n"
        out += f"\n#[async_trait]\nimpl {group_to_trait[i]} for PythonStore {{\n"
    
    out += process_method(m) + "\n"

out += "}\n"

with open("src/backend/python_store.rs", "w") as f:
    f.write(out)
