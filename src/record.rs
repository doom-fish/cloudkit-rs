use std::collections::BTreeMap;
use std::ffi::c_char;
use std::ops::{BitOr, BitOrAssign};
use std::path::{Path, PathBuf};

use crate::error::CloudKitError;
use crate::ffi;
use crate::private::{
    cstring_from_str, error_from_status, parse_json_ptr, CKAssetPayload, CKRecordIDPayload,
    CKRecordPayload, CKRecordValuePayload, CKRecordZoneIDPayload, CKRecordZonePayload,
    CKReferencePayload, RecordValueKind,
};
use crate::reference_utility::{CKReference, CKReferenceAction};

const DEFAULT_ZONE_NAME: &str = "_defaultZone";
const DEFAULT_OWNER_NAME: &str = "__defaultOwner__";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CKRecordZoneCapabilities(u64);

impl CKRecordZoneCapabilities {
    pub const FETCH_CHANGES: Self = Self(1 << 0);
    pub const ATOMIC: Self = Self(1 << 1);
    pub const SHARING: Self = Self(1 << 2);
    pub const ZONE_WIDE_SHARING: Self = Self(1 << 3);

    pub const fn new(bits: u64) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl BitOr for CKRecordZoneCapabilities {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for CKRecordZoneCapabilities {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKRecordZoneEncryptionScope {
    PerRecord,
    PerZone,
    Unknown(i32),
}

impl CKRecordZoneEncryptionScope {
    pub(crate) const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::PerRecord,
            1 => Self::PerZone,
            other => Self::Unknown(other),
        }
    }

    pub(crate) const fn to_raw(self) -> i32 {
        match self {
            Self::PerRecord => 0,
            Self::PerZone => 1,
            Self::Unknown(raw) => raw,
        }
    }
}

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
    Reference(CKReference),
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

impl From<CKReference> for RecordValue {
    fn from(value: CKReference) -> Self {
        Self::Reference(value)
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
            RecordValueKind::Reference => Self::Reference(CKReference::from_payload(
                payload.reference_value.unwrap_or_else(|| CKReferencePayload {
                    record_id: CKRecordID::new(String::new()).to_payload(),
                    action: CKReferenceAction::None.to_raw(),
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
                reference_value: None,
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
                reference_value: None,
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
                reference_value: None,
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
                reference_value: None,
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
                reference_value: None,
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
                reference_value: None,
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
                reference_value: None,
                array_value: None,
            },
            Self::Reference(reference) => CKRecordValuePayload {
                kind: RecordValueKind::Reference,
                string_value: None,
                int_value: None,
                double_value: None,
                bool_value: None,
                bytes_value: None,
                date_value: None,
                asset_value: None,
                reference_value: Some(reference.to_payload()),
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
                reference_value: None,
                array_value: Some(values.iter().map(Self::to_payload).collect()),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKRecordZone {
    zone_id: CKRecordZoneID,
    capabilities: CKRecordZoneCapabilities,
    share: Option<CKReference>,
    encryption_scope: Option<CKRecordZoneEncryptionScope>,
}

impl CKRecordZone {
    pub fn new(zone_name: impl Into<String>) -> Self {
        Self {
            zone_id: CKRecordZoneID::new(zone_name, DEFAULT_OWNER_NAME),
            capabilities: CKRecordZoneCapabilities::default(),
            share: None,
            encryption_scope: Some(CKRecordZoneEncryptionScope::PerRecord),
        }
    }

    pub fn with_zone_id(zone_id: CKRecordZoneID) -> Self {
        Self {
            zone_id,
            capabilities: CKRecordZoneCapabilities::default(),
            share: None,
            encryption_scope: Some(CKRecordZoneEncryptionScope::PerRecord),
        }
    }

    pub fn default_zone() -> Self {
        Self::with_zone_id(CKRecordZoneID::default_zone())
    }

    pub fn zone_id(&self) -> &CKRecordZoneID {
        &self.zone_id
    }

    pub const fn capabilities(&self) -> CKRecordZoneCapabilities {
        self.capabilities
    }

    pub const fn share(&self) -> Option<&CKReference> {
        self.share.as_ref()
    }

    pub const fn encryption_scope(&self) -> Option<CKRecordZoneEncryptionScope> {
        self.encryption_scope
    }

    pub(crate) fn from_payload(payload: CKRecordZonePayload) -> Self {
        Self {
            zone_id: CKRecordZoneID::from_payload(payload.zone_id),
            capabilities: CKRecordZoneCapabilities::new(payload.capabilities),
            share: payload.share.map(CKReference::from_payload),
            encryption_scope: payload
                .encryption_scope
                .map(CKRecordZoneEncryptionScope::from_raw),
        }
    }

    pub(crate) fn to_payload(&self) -> CKRecordZonePayload {
        CKRecordZonePayload {
            zone_id: self.zone_id.to_payload(),
            capabilities: self.capabilities.bits(),
            share: self.share.as_ref().map(CKReference::to_payload),
            encryption_scope: self.encryption_scope.map(CKRecordZoneEncryptionScope::to_raw),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKRecord {
    record_type: String,
    record_id: CKRecordID,
    fields: BTreeMap<String, RecordValue>,
    encoded_system_fields: Vec<u8>,
    record_change_tag: Option<String>,
    creator_user_record_id: Option<CKRecordID>,
    creation_date: Option<String>,
    last_modified_user_record_id: Option<CKRecordID>,
    modification_date: Option<String>,
    parent: Option<CKReference>,
    share: Option<CKReference>,
    changed_keys: Vec<String>,
    all_tokens: Vec<String>,
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

    pub fn with_record_id(record_type: impl Into<String>, record_id: CKRecordID) -> Self {
        Self {
            record_type: record_type.into(),
            record_id,
            fields: BTreeMap::new(),
            encoded_system_fields: Vec::new(),
            record_change_tag: None,
            creator_user_record_id: None,
            creation_date: None,
            last_modified_user_record_id: None,
            modification_date: None,
            parent: None,
            share: None,
            changed_keys: Vec::new(),
            all_tokens: Vec::new(),
        }
    }

    pub fn with_zone(record_type: impl Into<String>, zone_id: CKRecordZoneID) -> Self {
        Self::with_record_id(record_type, CKRecordID::with_zone(String::new(), zone_id))
    }

    pub fn record_type(&self) -> &str {
        &self.record_type
    }

    pub fn record_id(&self) -> &CKRecordID {
        &self.record_id
    }

    pub fn record_change_tag(&self) -> Option<&str> {
        self.record_change_tag.as_deref()
    }

    pub const fn creator_user_record_id(&self) -> Option<&CKRecordID> {
        self.creator_user_record_id.as_ref()
    }

    pub fn creation_date(&self) -> Option<&str> {
        self.creation_date.as_deref()
    }

    pub const fn last_modified_user_record_id(&self) -> Option<&CKRecordID> {
        self.last_modified_user_record_id.as_ref()
    }

    pub fn modification_date(&self) -> Option<&str> {
        self.modification_date.as_deref()
    }

    pub const fn parent(&self) -> Option<&CKReference> {
        self.parent.as_ref()
    }

    pub const fn share(&self) -> Option<&CKReference> {
        self.share.as_ref()
    }

    pub fn changed_keys(&self) -> &[String] {
        &self.changed_keys
    }

    pub fn all_tokens(&self) -> &[String] {
        &self.all_tokens
    }

    pub fn object(&self, key: &str) -> Option<&RecordValue> {
        self.fields.get(key)
    }

    pub fn set_object<V>(&mut self, key: &str, value: V)
    where
        V: Into<RecordValue>,
    {
        self.fields.insert(key.into(), value.into());
        self.mark_changed_key(key);
    }

    pub fn remove_object(&mut self, key: &str) -> Option<RecordValue> {
        let removed = self.fields.remove(key);
        if removed.is_some() {
            self.mark_changed_key(key);
        }
        removed
    }

    pub fn all_keys(&self) -> Vec<String> {
        self.fields.keys().cloned().collect()
    }

    pub fn encoded_system_fields(&self) -> &[u8] {
        &self.encoded_system_fields
    }

    pub fn set_parent_reference(&mut self, reference: CKReference) {
        self.parent = Some(reference);
    }

    pub fn set_parent_reference_from_record(&mut self, parent_record: &CKRecord) {
        self.set_parent_reference(CKReference::new(
            parent_record.record_id().clone(),
            CKReferenceAction::None,
        ));
    }

    pub fn set_parent_reference_from_record_id(&mut self, parent_record_id: CKRecordID) {
        self.set_parent_reference(CKReference::new(parent_record_id, CKReferenceAction::None));
    }

    pub fn clear_parent_reference(&mut self) {
        self.parent = None;
    }

    fn mark_changed_key(&mut self, key: &str) {
        if !self.changed_keys.iter().any(|changed| changed == key) {
            self.changed_keys.push(key.into());
        }
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
            record_change_tag: payload.record_change_tag,
            creator_user_record_id: payload.creator_user_record_id.map(CKRecordID::from_payload),
            creation_date: payload.creation_date,
            last_modified_user_record_id: payload
                .last_modified_user_record_id
                .map(CKRecordID::from_payload),
            modification_date: payload.modification_date,
            parent: payload.parent.map(CKReference::from_payload),
            share: payload.share.map(CKReference::from_payload),
            changed_keys: payload.changed_keys,
            all_tokens: payload.all_tokens,
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
            record_change_tag: self.record_change_tag.clone(),
            creator_user_record_id: self
                .creator_user_record_id
                .as_ref()
                .map(CKRecordID::to_payload),
            creation_date: self.creation_date.clone(),
            last_modified_user_record_id: self
                .last_modified_user_record_id
                .as_ref()
                .map(CKRecordID::to_payload),
            modification_date: self.modification_date.clone(),
            parent: self.parent.as_ref().map(CKReference::to_payload),
            share: self.share.as_ref().map(CKReference::to_payload),
            changed_keys: self.changed_keys.clone(),
            all_tokens: self.all_tokens.clone(),
        }
    }
}

pub trait CKRecordKeyValueSetting {
    fn object_for_key(&self, key: &str) -> Option<&RecordValue>;
    fn set_object_for_key<V>(&mut self, key: &str, value: V)
    where
        V: Into<RecordValue>;
    fn object_for_keyed_subscript(&self, key: &str) -> Option<&RecordValue>;
    fn set_object_for_keyed_subscript<V>(&mut self, key: &str, value: V)
    where
        V: Into<RecordValue>;
    fn all_keys(&self) -> Vec<String>;
    fn changed_keys(&self) -> &[String];
}

impl CKRecordKeyValueSetting for CKRecord {
    fn object_for_key(&self, key: &str) -> Option<&RecordValue> {
        self.object(key)
    }

    fn set_object_for_key<V>(&mut self, key: &str, value: V)
    where
        V: Into<RecordValue>,
    {
        self.set_object(key, value);
    }

    fn object_for_keyed_subscript(&self, key: &str) -> Option<&RecordValue> {
        self.object(key)
    }

    fn set_object_for_keyed_subscript<V>(&mut self, key: &str, value: V)
    where
        V: Into<RecordValue>,
    {
        self.set_object(key, value);
    }

    fn all_keys(&self) -> Vec<String> {
        self.all_keys()
    }

    fn changed_keys(&self) -> &[String] {
        self.changed_keys()
    }
}
