#[pyclass]
pub struct EvPushNameUpdateData {
    #[pyo3(get)]
    jid: Py<JID>,
    #[pyo3(get)]
    message: Py<MessageInfo>,
    #[pyo3(get)]
    old_push_name: String,
    #[pyo3(get)]
    new_push_name: String,
}

#[pyclass]
pub struct EvPushNameUpdate {
    inner: Box<wacore::types::events::PushNameUpdate>,
    data_cache: OnceLock<Py<EvPushNameUpdateData>>,
}

impl EvPushNameUpdate {
    pub fn new(inner: wacore::types::events::PushNameUpdate) -> Self {
        Self {
            inner: Box::new(inner),
            data_cache: OnceLock::new(),
        }
    }
}

impl From<wacore::types::events::PushNameUpdate> for EvPushNameUpdate {
    fn from(event: wacore::types::events::PushNameUpdate) -> Self {
        EvPushNameUpdate::new(event)
    }
}

#[pymethods]
impl EvPushNameUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<EvPushNameUpdateData>> {
        if let Some(cached) = self.data_cache.get() {
            Ok(cached.clone_ref(py))
        } else {
            let message = MessageInfo::from((*self.inner.message).clone());
            let data = EvPushNameUpdateData {
                jid: Py::new(py, JID::from(self.inner.jid.clone())).unwrap(),
                message: Py::new(py, message)?,
                old_push_name: self.inner.old_push_name.clone(),
                new_push_name: self.inner.new_push_name.clone(),
            };
            let py_data = Py::new(py, data)?;
            self.data_cache.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}


#[pyclass]
pub struct EvSelfPushNameUpdated {
    #[pyo3(get)]
    from_server: bool,
    #[pyo3(get)]
    old_name: String,
    #[pyo3(get)]
    new_name: String,
}
impl From<wacore::types::events::SelfPushNameUpdated> for EvSelfPushNameUpdated {
    fn from(event: wacore::types::events::SelfPushNameUpdated) -> Self {
        EvSelfPushNameUpdated { from_server: event.from_server, old_name: event.old_name, new_name: event.new_name }
    }
}

#[pyclass]
pub struct PinUpdatedata {
    #[pyo3(get)]
    jid: Py<JID>,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
    #[pyo3(get)]
    pinned: Option<bool>,
    #[pyo3(get)]
    from_full_sync: bool,
}
impl PinUpdatedata {
    pub fn new(jid: Jid, timestamp: DateTime<Utc>, action: waproto::whatsapp::sync_action_value::PinAction, from_full_sync: bool) -> Self {
        Python::attach(|py| {
            Self {
                jid: Py::new(py, JID::from(jid)).unwrap(),
                timestamp: PyDateTime::from_timestamp(py, timestamp.timestamp() as f64, None).unwrap().into(),
                pinned: action.pinned,
                from_full_sync,
            }
        })
    }
}

#[pyclass]
pub struct EvPinUpdate {
    inner: Box<wacore::types::events::PinUpdate>,
    data_cached: OnceLock<Py<PinUpdatedata>>,
}
impl EvPinUpdate {
    pub fn new(inner: wacore::types::events::PinUpdate) -> Self {
        Self { inner: Box::new(inner), data_cached: OnceLock::new() }
    }
}
impl From<wacore::types::events::PinUpdate> for EvPinUpdate {
    fn from(event: wacore::types::events::PinUpdate) -> Self {
        EvPinUpdate::new(event)
    }
}
#[pymethods]
impl EvPinUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<PinUpdatedata>> {
        if let Some(ref data) = self.data_cached.get() {
            Ok(data.clone_ref(py))
        } else {
            let new_data = PinUpdatedata::new(self.inner.jid.clone(), self.inner.timestamp, (*self.inner.action).clone(), self.inner.from_full_sync);
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}



#[pyclass]
pub struct MuteUpdateData {
    #[pyo3(get)]
    jid: Py<JID>,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
    action: Box<waproto::whatsapp::sync_action_value::MuteAction>,
    action_cached: OnceLock<Py<PyAny>>,
    #[pyo3(get)]
    from_full_sync: bool,
}

impl MuteUpdateData {
    pub fn new(jid: JID, timestamp: DateTime<Utc>, action: waproto::whatsapp::sync_action_value::MuteAction, from_full_sync: bool) -> Self {
        Python::attach(|py| {
            Self {
                jid: Py::new(py, JID::from(jid)).unwrap(),
                timestamp: PyDateTime::from_timestamp(py, timestamp.timestamp() as f64, None).unwrap().into(),
                action: Box::new(action),
                action_cached: OnceLock::new(),
                from_full_sync,
            }
        })
    }
}

#[pymethods]
impl MuteUpdateData {
    #[getter]
    fn action(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if let Some(cached) = self.action_cached.get() {
            Ok(cached.clone_ref(py))
        } else {
            let proto_type = get_proto_mute_action_proto_type(py)?;
            let mut proto_bytes = Vec::new();
            self.action.encode(&mut proto_bytes).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to encode MuteAction proto: {}", e)))?;
            let parsed_proto = from_string_to_python_proto(py, proto_type, &proto_bytes)?;
            self.action_cached.set(parsed_proto.clone_ref(py)).ok();
            Ok(parsed_proto)
        }
    }
}

#[pyclass]
pub struct EvMuteUpdate {
    inner: Box<wacore::types::events::MuteUpdate>,
    data_cached: OnceLock<Py<MuteUpdateData>>,
}
impl EvMuteUpdate {
    pub fn new(inner: wacore::types::events::MuteUpdate) -> Self {
        Self { inner: Box::new(inner), data_cached: OnceLock::new() }
    }
}
#[pymethods]
impl EvMuteUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<MuteUpdateData>> {
        if let Some(ref data) = self.data_cached.get() {
            Ok(data.clone_ref(py))
        } else {
            let new_data = MuteUpdateData::new(self.inner.jid.clone().into(), self.inner.timestamp, (*self.inner.action).clone(), self.inner.from_full_sync);
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}
impl From<wacore::types::events::MuteUpdate> for EvMuteUpdate {
    fn from(event: wacore::types::events::MuteUpdate) -> Self {
        EvMuteUpdate::new(event)
    }
}

#[pyclass]
pub struct MarkChatAsReadUpdateData {
    #[pyo3(get)]
    jid: Py<JID>,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
    action: Box<waproto::whatsapp::sync_action_value::MarkChatAsReadAction>,
    action_cached: OnceLock<Py<PyAny>>,
    #[pyo3(get)]
    from_full_sync: bool,
}
impl MarkChatAsReadUpdateData {
    pub fn new(jid: JID, timestamp: DateTime<Utc>, action: waproto::whatsapp::sync_action_value::MarkChatAsReadAction, from_full_sync: bool) -> Self {
        Python::attach(|py| {
            Self {
                jid: Py::new(py, JID::from(jid)).unwrap(),
                timestamp: PyDateTime::from_timestamp(py, timestamp.timestamp() as f64, None).unwrap().into(),
                action: Box::new(action),
                action_cached: OnceLock::new(),
                from_full_sync,
            }
        })
    }
}
#[pymethods]
impl MarkChatAsReadUpdateData {
    #[getter]
    fn action(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if let Some(cached) = self.action_cached.get() {
            Ok(cached.clone_ref(py))
        } else {
            let proto_type = get_proto_mark_chat_as_read_action_proto_type(py)?;
            let mut proto_bytes = Vec::new();
            self.action.encode(&mut proto_bytes).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to encode MarkChatAsReadAction proto: {}", e)))?;
            let parsed_proto = from_string_to_python_proto(py, proto_type, &proto_bytes)?;
            self.action_cached.set(parsed_proto.clone_ref(py)).ok();
            Ok(parsed_proto)
        }
    }
}
#[pyclass]
pub struct EvMarkChatAsReadUpdate {
    inner: Box<wacore::types::events::MarkChatAsReadUpdate>,
    data_cached: OnceLock<Py<MarkChatAsReadUpdateData>>,
}
impl EvMarkChatAsReadUpdate {
    pub fn new(inner: wacore::types::events::MarkChatAsReadUpdate) -> Self {
        Self { inner: Box::new(inner), data_cached: OnceLock::new() }
    }
}
impl From<wacore::types::events::MarkChatAsReadUpdate> for EvMarkChatAsReadUpdate {
    fn from(event: wacore::types::events::MarkChatAsReadUpdate) -> Self {
        EvMarkChatAsReadUpdate::new(event)
    }
}
#[pymethods]
impl EvMarkChatAsReadUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> PyResult<Py<MarkChatAsReadUpdateData>> {
        if let Some(ref data) = self.data_cached.get() {
            Ok(data.clone_ref(py))
        } else {
            let new_data = MarkChatAsReadUpdateData::new(JID::from(self.inner.jid.clone()), self.inner.timestamp, (*self.inner.action).clone(), self.inner.from_full_sync);
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            Ok(py_data)
        }
    }
}



#[pyclass]
pub struct EvHistorySync{
    inner: Box<waproto::whatsapp::HistorySync>,
    proto_cached: OnceLock<Py<PyAny>>,
}
impl EvHistorySync {
    pub fn new(inner: waproto::whatsapp::HistorySync) -> Self {
        Self { inner: Box::new(inner), proto_cached: OnceLock::new() }
    }
}
impl From<waproto::whatsapp::HistorySync> for EvHistorySync {
    fn from(event: waproto::whatsapp::HistorySync) -> Self {
        EvHistorySync::new(event)
    }
}
#[pymethods]
impl EvHistorySync {
    #[getter]
    fn proto(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        if let Some(ref proto) = self.proto_cached.get() {
            Ok(proto.clone_ref(py))
        } else {
            let proto_type = get_proto_history_sync_proto_type(py)?;
            let mut proto_bytes = Vec::new();
            self.inner.encode(&mut proto_bytes).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to encode HistorySync proto: {}", e)))?;
            let parsed_proto = from_string_to_python_proto(py, proto_type, &proto_bytes)?;
            self.proto_cached.set(parsed_proto.clone_ref(py)).ok();
            Ok(parsed_proto)
        }
    }
}
#[pyclass]
pub struct OfflineSyncData {
    #[pyo3(get)]
    total: i32,
    #[pyo3(get)]
    app_data_changes: i32,
    #[pyo3(get)]
    messages: i32,
    #[pyo3(get)]
    notifications: i32,
    #[pyo3(get)]
    receipts: i32,
}
impl OfflineSyncData {
    pub fn new(total: i32, app_data_changes: i32, messages: i32, notifications: i32, receipts: i32) -> Self {
        Self { total, app_data_changes, messages, notifications, receipts }
    }
}
#[pyclass]
pub struct EvOfflineSyncPreview {
    inner: Box<wacore::types::events::OfflineSyncPreview>,
    data_cached: OnceLock<Py<OfflineSyncData>>,
}
#[pymethods]
impl EvOfflineSyncPreview {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> Py<OfflineSyncData> {
        if let Some(ref data) = self.data_cached.get() {
            data.clone_ref(py)
        } else {
            let new_data = OfflineSyncData::new(self.inner.total, self.inner.app_data_changes, self.inner.messages, self.inner.notifications, self.inner.receipts);
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            py_data
        }
    }
}
impl EvOfflineSyncPreview {
    pub fn new(inner: wacore::types::events::OfflineSyncPreview) -> Self {
        Self { inner: Box::new(inner), data_cached: OnceLock::new() }
    }
}
impl From<wacore::types::events::OfflineSyncPreview> for EvOfflineSyncPreview {
    fn from(event: wacore::types::events::OfflineSyncPreview) -> Self {
        EvOfflineSyncPreview::new(event)
    }
}
#[pyclass]
pub struct OfflineSyncCompletedData {
    #[pyo3(get)]
    count: i32,
}
impl OfflineSyncCompletedData {
    pub fn new(count: i32) -> Self {
        Self { count }
    }
}

#[pyclass]
pub struct EvOfflineSyncCompleted{
    inner: Box<wacore::types::events::OfflineSyncCompleted>,
    data_cached: OnceLock<Py<OfflineSyncCompletedData>>,
}

impl EvOfflineSyncCompleted {
    pub fn new(inner: wacore::types::events::OfflineSyncCompleted) -> Self {
        Self { inner: Box::new(inner), data_cached: OnceLock::new() }
    }
}
impl From<wacore::types::events::OfflineSyncCompleted> for EvOfflineSyncCompleted {
    fn from(event: wacore::types::events::OfflineSyncCompleted) -> Self {
        EvOfflineSyncCompleted::new(event)
    }
}
#[pymethods]
impl EvOfflineSyncCompleted {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> Py<OfflineSyncCompletedData> {
        if let Some(ref data) = self.data_cached.get() {
            data.clone_ref(py)
        } else {
            let new_data = OfflineSyncCompletedData::new(self.inner.count);
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            py_data
        }
    }
}

#[pyclass]
pub enum DeviceListUpdateType {
    Added,
    Removed,
    Updated
}
#[pyclass]
pub struct DeviceNottificationInfo {
    #[pyo3(get)]
    device_id: u32,
    #[pyo3(get)]
    key_index: Option<u32>,
}
#[pyclass]
pub struct DeviceListUpdateData {
    #[pyo3(get)]
    user: Py<JID>,
    #[pyo3(get)]
    lid_user: Option<Py<JID>>,
    #[pyo3(get)]
    update_type: Py<DeviceListUpdateType>,
    #[pyo3(get)]
    devices: Vec<Py<DeviceNottificationInfo>>,
    #[pyo3(get)]
    key_index: Option<Py<KeyIndexInfo>>,
    #[pyo3(get)]
    contact_hash: Option<String>,
}

#[pyclass]
pub struct EvDeviceListUpdate {
    inner: Box<wacore::types::events::DeviceListUpdate>,
    data_cached: OnceLock<Py<DeviceListUpdateData>>,
}
impl EvDeviceListUpdate {
    pub fn new(inner: wacore::types::events::DeviceListUpdate) -> Self {
        Self { inner: Box::new(inner), data_cached: OnceLock::new() }
    }
}
impl From<wacore::types::events::DeviceListUpdate> for EvDeviceListUpdate {
    fn from(event: wacore::types::events::DeviceListUpdate) -> Self {
        EvDeviceListUpdate::new(event)
    }
}
#[pymethods]
impl EvDeviceListUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> Py<DeviceListUpdateData> {
        if let Some(data) = self.data_cached.get() {
            data.clone_ref(py)
        } else {
            let update_type = match self.inner.update_type {
                wacore::types::events::DeviceListUpdateType::Add => DeviceListUpdateType::Added,
                wacore::types::events::DeviceListUpdateType::Remove => DeviceListUpdateType::Removed,
                wacore::types::events::DeviceListUpdateType::Update => DeviceListUpdateType::Updated,
            };
            let devices = self.inner.devices.iter().map(|d| {
                Py::new(py, DeviceNottificationInfo { device_id: d.device_id, key_index: d.key_index }).unwrap()
            }).collect();
            let key_index = self.inner.key_index.clone().map(|k| Py::new(py, KeyIndexInfo::new(k.timestamp, k.signed_bytes)).unwrap());
            let new_data = DeviceListUpdateData {
                user: Py::new(py, JID::from(self.inner.user.clone())).unwrap(),
                lid_user: self.inner.lid_user.clone().map(|u| Py::new(py, JID::from(u)).unwrap()),
                update_type: Py::new(py, update_type).unwrap(),
                devices,
                key_index,
                contact_hash: self.inner.contact_hash.clone(),
            };
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            py_data
        }
    }
}
#[pyclass]
pub enum BusinessStatusUpdateType {
    RemovedAsBusiness,
    VerifiedNameChanged,
    ProfileUpdated,
    ProductsUpdated,
    CollectionsUpdated,
    SubscriptionsUpdated,
    Unknown,
}
#[pyclass]
pub struct BusinessStatusUpdateData {
    #[pyo3(get)]
    jid: Py<JID>,
    #[pyo3(get)]
    update_type: Py<BusinessStatusUpdateType>,
    #[pyo3(get)]
    timestamp: Py<PyDateTime>,
    #[pyo3(get)]
    target_jid: Option<Py<JID>>,
    #[pyo3(get)]
    hash: Option<String>,
    #[pyo3(get)]
    product_ids: Vec<String>,
    #[pyo3(get)]
    collection_ids: Vec<String>,
    #[pyo3(get)]
    subscriptions: Vec<Py<BusinessSubscription>>,
}
impl BusinessStatusUpdateData {
    pub fn new(jid: JID, update_type: BusinessStatusUpdateType, timestamp: i64, target_jid: Option<JID>, hash: Option<String>, product_ids: Vec<String>, collection_ids: Vec<String>, subscriptions: Vec<BusinessSubscription>) -> Self {
        Python::attach(|py| {
            Self {
                jid: Py::new(py, JID::from(jid)).unwrap(),
                update_type: Py::new(py, update_type).unwrap(),
                timestamp: PyDateTime::from_timestamp(py, timestamp as f64, None).unwrap().into(),
                target_jid: target_jid.map(|j| Py::new(py, JID::from(j)).unwrap()),
                hash,
                product_ids,
                collection_ids,
                subscriptions: subscriptions.into_iter().map(|s| Py::new(py, s).unwrap()).collect(),
            }
        })
    }
}

#[pyclass]
pub struct EvBusinessStatusUpdate{
    inner: Box<wacore::types::events::BusinessStatusUpdate>,
    data_cached: OnceLock<Py<BusinessStatusUpdateData>>,
}
impl EvBusinessStatusUpdate {
    pub fn new(inner: wacore::types::events::BusinessStatusUpdate) -> Self {
        Self { inner: Box::new(inner), data_cached: OnceLock::new() }
    }
}
impl From<wacore::types::events::BusinessStatusUpdate> for EvBusinessStatusUpdate {
    fn from(event: wacore::types::events::BusinessStatusUpdate) -> Self {
        EvBusinessStatusUpdate::new(event)
    }
}
#[pymethods]
impl EvBusinessStatusUpdate {
    #[getter]
    fn data(&mut self, py: Python<'_>) -> Py<BusinessStatusUpdateData> {
        if let Some(ref data) = self.data_cached.get() {
            data.clone_ref(py)
        } else {
            let update_type = match self.inner.update_type {
                wacore::types::events::BusinessUpdateType::RemovedAsBusiness => BusinessStatusUpdateType::RemovedAsBusiness,
                wacore::types::events::BusinessUpdateType::VerifiedNameChanged => BusinessStatusUpdateType::VerifiedNameChanged,
                wacore::types::events::BusinessUpdateType::ProfileUpdated => BusinessStatusUpdateType::ProfileUpdated,
                wacore::types::events::BusinessUpdateType::ProductsUpdated => BusinessStatusUpdateType::ProductsUpdated,
                wacore::types::events::BusinessUpdateType::CollectionsUpdated => BusinessStatusUpdateType::CollectionsUpdated,
                wacore::types::events::BusinessUpdateType::SubscriptionsUpdated => BusinessStatusUpdateType::SubscriptionsUpdated,
                wacore::types::events::BusinessUpdateType::Unknown => BusinessStatusUpdateType::Unknown,
            };
            let new_data = BusinessStatusUpdateData::new(self.inner.jid.clone().into(), update_type, self.inner.timestamp, self.inner.target_jid.clone().map(|j| j.into()), self.inner.hash.clone(), self.inner.product_ids.clone(), self.inner.collection_ids.clone(), self.inner.subscriptions.iter().map(|s| s.clone().into()).collect());
            let py_data = Py::new(py, new_data).unwrap();
            self.data_cached.set(py_data.clone_ref(py)).ok();
            py_data
        }
    }
}
