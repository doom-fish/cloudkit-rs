use core::ffi::{c_char, c_void};
use std::collections::BTreeMap;
use std::ffi::{CStr, CString};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::{CloudKitError, ErrorPayload};
use crate::ffi;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKRecordZoneIDPayload {
    pub zone_name: String,
    pub owner_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKRecordIDPayload {
    pub record_name: String,
    pub zone_id: CKRecordZoneIDPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKAssetPayload {
    pub file_url: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum RecordValueKind {
    String,
    Int,
    Double,
    Bool,
    Bytes,
    Date,
    Asset,
    Array,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKRecordValuePayload {
    pub kind: RecordValueKind,
    pub string_value: Option<String>,
    pub int_value: Option<i64>,
    pub double_value: Option<f64>,
    pub bool_value: Option<bool>,
    pub bytes_value: Option<Vec<u8>>,
    pub date_value: Option<String>,
    pub asset_value: Option<CKAssetPayload>,
    pub array_value: Option<Vec<CKRecordValuePayload>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKRecordPayload {
    pub record_type: String,
    pub record_id: CKRecordIDPayload,
    pub fields: BTreeMap<String, CKRecordValuePayload>,
    pub encoded_system_fields: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SortDescriptorPayload {
    pub key: String,
    pub ascending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKQueryPayload {
    pub record_type: String,
    pub predicate_format: String,
    pub sort_descriptors: Vec<SortDescriptorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKModifyRecordsOperationPayload {
    pub records_to_save: Vec<CKRecordPayload>,
    pub record_ids_to_delete: Vec<CKRecordIDPayload>,
    pub save_policy: i32,
    pub atomic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKRecordSaveResultPayload {
    pub record_id: CKRecordIDPayload,
    pub record: Option<CKRecordPayload>,
    pub error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKRecordDeleteResultPayload {
    pub record_id: CKRecordIDPayload,
    pub error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKModifyRecordsResultPayload {
    pub saved_records: Vec<CKRecordPayload>,
    pub deleted_record_ids: Vec<CKRecordIDPayload>,
    pub save_results: Vec<CKRecordSaveResultPayload>,
    pub delete_results: Vec<CKRecordDeleteResultPayload>,
    pub operation_error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKQueryOperationPayload {
    pub query: CKQueryPayload,
    pub zone_id: Option<CKRecordZoneIDPayload>,
    pub desired_keys: Option<Vec<String>>,
    pub results_limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKQueryMatchResultPayload {
    pub record_id: CKRecordIDPayload,
    pub record: Option<CKRecordPayload>,
    pub error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKQueryOperationResultPayload {
    pub records: Vec<CKRecordPayload>,
    pub matches: Vec<CKQueryMatchResultPayload>,
    pub cursor_returned: bool,
    pub operation_error: Option<ErrorPayload>,
}

pub(crate) fn cstring_from_str(value: &str, context: &str) -> Result<CString, CloudKitError> {
    CString::new(value).map_err(|error| {
        CloudKitError::bridge(
            -1,
            format!("{context} contains an interior NUL byte: {error}"),
        )
    })
}

pub(crate) fn optional_cstring_from_str(
    value: Option<&str>,
    context: &str,
) -> Result<Option<CString>, CloudKitError> {
    value
        .map(|value| cstring_from_str(value, context))
        .transpose()
}

pub(crate) fn json_cstring<T: Serialize>(
    value: &T,
    context: &str,
) -> Result<CString, CloudKitError> {
    let json = serde_json::to_string(value).map_err(|error| {
        CloudKitError::bridge(-1, format!("failed to encode {context} as JSON: {error}"))
    })?;
    cstring_from_str(&json, context)
}

pub(crate) fn opt_cstring_ptr(value: &Option<CString>) -> *const c_char {
    value
        .as_ref()
        .map_or(core::ptr::null(), |value| value.as_c_str().as_ptr())
}

pub(crate) unsafe fn take_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let string = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    ffi::ck_string_free(ptr);
    Some(string)
}

pub(crate) fn parse_json_str<T: DeserializeOwned>(
    json: &str,
    context: &str,
) -> Result<T, CloudKitError> {
    serde_json::from_str(json).map_err(|error| {
        CloudKitError::bridge(
            -1,
            format!("failed to parse {context} JSON: {error}; payload={json}"),
        )
    })
}

pub(crate) unsafe fn parse_json_ptr<T: DeserializeOwned>(
    ptr: *mut c_char,
    context: &str,
) -> Result<T, CloudKitError> {
    let json = take_string(ptr)
        .ok_or_else(|| CloudKitError::bridge(-1, format!("missing JSON payload for {context}")))?;
    parse_json_str(&json, context)
}

pub(crate) unsafe fn parse_error_ptr(ptr: *mut c_char) -> CloudKitError {
    if ptr.is_null() {
        return CloudKitError::bridge(-2, "CloudKit bridge returned an error without payload");
    }
    let json = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    ffi::ck_string_free(ptr);
    parse_error_json_str(&json)
}

pub(crate) unsafe fn parse_borrowed_error_ptr(ptr: *const c_char) -> CloudKitError {
    if ptr.is_null() {
        return CloudKitError::bridge(-2, "CloudKit bridge returned an error without payload");
    }
    let json = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    parse_error_json_str(&json)
}

pub(crate) fn parse_error_json_str(json: &str) -> CloudKitError {
    match serde_json::from_str::<ErrorPayload>(json) {
        Ok(payload) => CloudKitError::from_payload(payload),
        Err(error) => CloudKitError::bridge(
            -1,
            format!("failed to parse CloudKit error payload: {error}; payload={json}"),
        ),
    }
}

pub(crate) unsafe fn error_from_status(status: i32, err_msg: *mut c_char) -> CloudKitError {
    if !err_msg.is_null() {
        return parse_error_ptr(err_msg);
    }
    let message = match status {
        ffi::status::INVALID_ARGUMENT => "invalid argument",
        ffi::status::TIMED_OUT => "timed out waiting for CloudKit",
        _ => "CloudKit bridge failure",
    };
    CloudKitError::bridge(i64::from(status), message)
}

pub(crate) fn box_closure<F>(closure: F) -> *mut c_void
where
    F: Send + 'static,
{
    Box::into_raw(Box::new(closure)).cast::<c_void>()
}
