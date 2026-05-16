use std::collections::BTreeMap;
use std::ffi::c_char;
use std::path::{Path, PathBuf};

use crate::error::CloudKitError;
use crate::ffi;
use crate::private::{
    cstring_from_str, error_from_status, parse_json_ptr, CKAssetPayload, CKRecordIDPayload,
    CKRecordPayload, CKRecordValuePayload, CKRecordZoneIDPayload, RecordValueKind,
};

const DEFAULT_ZONE_NAME: &str = "_defaultZone";
const DEFAULT_OWNER_NAME: &str = "__defaultOwner__";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKRecordZoneID {
    zone_name: String,
    owner_name: String,
}

impl CKRecordZoneID {
    pub fn new(zone_name: impl Into<String>, owner_name: impl Into<String>) -> Self {
        Self {
            zone_name: zone_name.into(),
            owner_name: owner_name.into(),
        }
    }

    pub fn default_zone() -> Self {
        Self::new(DEFAULT_ZONE_NAME, DEFAULT_OWNER_NAME)
    }

    pub fn zone_name(&self) -> &str {
        &self.zone_name
    }

    pub fn owner_name(&self) -> &str {
        &self.owner_name
    }

    pub(crate) fn from_payload(payload: CKRecordZoneIDPayload) -> Self {
        Self::new(payload.zone_name, payload.owner_name)
    }

    pub(crate) fn to_payload(&self) -> CKRecordZoneIDPayload {
        CKRecordZoneIDPayload {
            zone_name: self.zone_name.clone(),
            owner_name: self.owner_name.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKRecordID {
    record_name: String,
    zone_id: CKRecordZoneID,
}

impl CKRecordID {
    pub fn new(record_name: impl Into<String>) -> Self {
        Self::with_zone(record_name, CKRecordZoneID::default_zone())
    }

    pub fn with_zone(record_name: impl Into<String>, zone_id: CKRecordZoneID) -> Self {
        Self {
            record_name: record_name.into(),
            zone_id,
        }
    }

    pub fn record_name(&self) -> &str {
        &self.record_name
    }

    pub fn zone_id(&self) -> &CKRecordZoneID {
        &self.zone_id
    }

    pub(crate) fn from_payload(payload: CKRecordIDPayload) -> Self {
        Self::with_zone(
            payload.record_name,
            CKRecordZoneID::from_payload(payload.zone_id),
        )
    }

    pub(crate) fn to_payload(&self) -> CKRecordIDPayload {
        CKRecordIDPayload {
            record_name: self.record_name.clone(),
            zone_id: self.zone_id.to_payload(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKAsset {
    file_url: PathBuf,
}

impl CKAsset {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            file_url: path.into(),
        }
    }

    pub fn file_url(&self) -> &Path {
        &self.file_url
    }

    pub(crate) fn from_payload(payload: CKAssetPayload) -> Self {
        Self::new(payload.file_url)
    }

    pub(crate) fn to_payload(&self) -> CKAssetPayload {
        CKAssetPayload {
            file_url: self.file_url.to_string_lossy().into_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum RecordValue {
    String(String),
    Int(i64),
    Double(f64),
    Bool(bool),
    Bytes(Vec<u8>),
    Date(String),
    Asset(CKAsset),
    Array(Vec<RecordValue>),
}

impl From<String> for RecordValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for RecordValue {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<bool> for RecordValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for RecordValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<i32> for RecordValue {
    fn from(value: i32) -> Self {
        Self::Int(i64::from(value))
    }
}

impl From<f64> for RecordValue {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl From<Vec<u8>> for RecordValue {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}

impl From<CKAsset> for RecordValue {
    fn from(value: CKAsset) -> Self {
        Self::Asset(value)
    }
}

impl From<Vec<RecordValue>> for RecordValue {
    fn from(value: Vec<RecordValue>) -> Self {
        Self::Array(value)
    }
}

impl RecordValue {
    pub(crate) fn from_payload(payload: CKRecordValuePayload) -> Self {
        match payload.kind {
            RecordValueKind::String => Self::String(payload.string_value.unwrap_or_default()),
            RecordValueKind::Int => Self::Int(payload.int_value.unwrap_or_default()),
            RecordValueKind::Double => Self::Double(payload.double_value.unwrap_or_default()),
            RecordValueKind::Bool => Self::Bool(payload.bool_value.unwrap_or_default()),
            RecordValueKind::Bytes => Self::Bytes(payload.bytes_value.unwrap_or_default()),
            RecordValueKind::Date => Self::Date(payload.date_value.unwrap_or_default()),
            RecordValueKind::Asset => Self::Asset(CKAsset::from_payload(
                payload.asset_value.unwrap_or(CKAssetPayload {
                    file_url: String::new(),
                }),
            )),
            RecordValueKind::Array => Self::Array(
                payload
                    .array_value
                    .unwrap_or_default()
                    .into_iter()
                    .map(Self::from_payload)
                    .collect(),
            ),
        }
    }

    pub(crate) fn to_payload(&self) -> CKRecordValuePayload {
        match self {
            Self::String(value) => CKRecordValuePayload {
                kind: RecordValueKind::String,
                string_value: Some(value.clone()),
                int_value: None,
                double_value: None,
                bool_value: None,
                bytes_value: None,
                date_value: None,
                asset_value: None,
                array_value: None,
            },
            Self::Int(value) => CKRecordValuePayload {
                kind: RecordValueKind::Int,
                string_value: None,
                int_value: Some(*value),
                double_value: None,
                bool_value: None,
                bytes_value: None,
                date_value: None,
                asset_value: None,
                array_value: None,
            },
            Self::Double(value) => CKRecordValuePayload {
                kind: RecordValueKind::Double,
                string_value: None,
                int_value: None,
                double_value: Some(*value),
                bool_value: None,
                bytes_value: None,
                date_value: None,
                asset_value: None,
                array_value: None,
            },
            Self::Bool(value) => CKRecordValuePayload {
                kind: RecordValueKind::Bool,
                string_value: None,
                int_value: None,
                double_value: None,
                bool_value: Some(*value),
                bytes_value: None,
                date_value: None,
                asset_value: None,
                array_value: None,
            },
            Self::Bytes(value) => CKRecordValuePayload {
                kind: RecordValueKind::Bytes,
                string_value: None,
                int_value: None,
                double_value: None,
                bool_value: None,
                bytes_value: Some(value.clone()),
                date_value: None,
                asset_value: None,
                array_value: None,
            },
            Self::Date(value) => CKRecordValuePayload {
                kind: RecordValueKind::Date,
                string_value: None,
                int_value: None,
                double_value: None,
                bool_value: None,
                bytes_value: None,
                date_value: Some(value.clone()),
                asset_value: None,
                array_value: None,
            },
            Self::Asset(asset) => CKRecordValuePayload {
                kind: RecordValueKind::Asset,
                string_value: None,
                int_value: None,
                double_value: None,
                bool_value: None,
                bytes_value: None,
                date_value: None,
                asset_value: Some(asset.to_payload()),
                array_value: None,
            },
            Self::Array(values) => CKRecordValuePayload {
                kind: RecordValueKind::Array,
                string_value: None,
                int_value: None,
                double_value: None,
                bool_value: None,
                bytes_value: None,
                date_value: None,
                asset_value: None,
                array_value: Some(values.iter().map(Self::to_payload).collect()),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKRecordZone {
    zone_id: CKRecordZoneID,
    capabilities: u64,
}

impl CKRecordZone {
    pub fn new(zone_name: impl Into<String>) -> Self {
        Self {
            zone_id: CKRecordZoneID::new(zone_name, DEFAULT_OWNER_NAME),
            capabilities: 0,
        }
    }

    pub fn default_zone() -> Self {
        Self {
            zone_id: CKRecordZoneID::default_zone(),
            capabilities: 0,
        }
    }

    pub fn zone_id(&self) -> &CKRecordZoneID {
        &self.zone_id
    }

    pub const fn capabilities(&self) -> u64 {
        self.capabilities
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKRecord {
    record_type: String,
    record_id: CKRecordID,
    fields: BTreeMap<String, RecordValue>,
    encoded_system_fields: Vec<u8>,
}

impl CKRecord {
    pub fn new(record_type: &str) -> Result<Self, CloudKitError> {
        let record_type = cstring_from_str(record_type, "record type")?;
        let mut out_json: *mut c_char = core::ptr::null_mut();
        let mut out_error: *mut c_char = core::ptr::null_mut();
        let status =
            unsafe { ffi::ck_record_create(record_type.as_ptr(), &mut out_json, &mut out_error) };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        let payload: CKRecordPayload = unsafe { parse_json_ptr(out_json, "record")? };
        Ok(Self::from_payload(payload))
    }

    pub fn record_type(&self) -> &str {
        &self.record_type
    }

    pub fn record_id(&self) -> &CKRecordID {
        &self.record_id
    }

    pub fn object(&self, key: &str) -> Option<&RecordValue> {
        self.fields.get(key)
    }

    pub fn set_object<V>(&mut self, key: &str, value: V)
    where
        V: Into<RecordValue>,
    {
        self.fields.insert(key.into(), value.into());
    }

    pub fn remove_object(&mut self, key: &str) -> Option<RecordValue> {
        self.fields.remove(key)
    }

    pub fn all_keys(&self) -> Vec<String> {
        self.fields.keys().cloned().collect()
    }

    pub fn encoded_system_fields(&self) -> &[u8] {
        &self.encoded_system_fields
    }

    pub(crate) fn from_payload(payload: CKRecordPayload) -> Self {
        Self {
            record_type: payload.record_type,
            record_id: CKRecordID::from_payload(payload.record_id),
            fields: payload
                .fields
                .into_iter()
                .map(|(key, value)| (key, RecordValue::from_payload(value)))
                .collect(),
            encoded_system_fields: payload.encoded_system_fields,
        }
    }

    pub(crate) fn to_payload(&self) -> CKRecordPayload {
        CKRecordPayload {
            record_type: self.record_type.clone(),
            record_id: self.record_id.to_payload(),
            fields: self
                .fields
                .iter()
                .map(|(key, value)| (key.clone(), value.to_payload()))
                .collect(),
            encoded_system_fields: self.encoded_system_fields.clone(),
        }
    }
}
