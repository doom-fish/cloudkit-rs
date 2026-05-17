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
    #[serde(rename = "zoneID")]
    pub zone_id: CKRecordZoneIDPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKReferencePayload {
    #[serde(rename = "recordID")]
    pub record_id: CKRecordIDPayload,
    pub action: u64,
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
    Reference,
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
    pub reference_value: Option<CKReferencePayload>,
    pub array_value: Option<Vec<CKRecordValuePayload>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKRecordPayload {
    pub record_type: String,
    #[serde(rename = "recordID")]
    pub record_id: CKRecordIDPayload,
    pub fields: BTreeMap<String, CKRecordValuePayload>,
    pub encoded_system_fields: Vec<u8>,
    pub record_change_tag: Option<String>,
    #[serde(rename = "creatorUserRecordID")]
    pub creator_user_record_id: Option<CKRecordIDPayload>,
    pub creation_date: Option<String>,
    #[serde(rename = "lastModifiedUserRecordID")]
    pub last_modified_user_record_id: Option<CKRecordIDPayload>,
    pub modification_date: Option<String>,
    pub parent: Option<CKReferencePayload>,
    pub share: Option<CKReferencePayload>,
    pub changed_keys: Vec<String>,
    pub all_tokens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKRecordZonePayload {
    #[serde(rename = "zoneID")]
    pub zone_id: CKRecordZoneIDPayload,
    pub capabilities: u64,
    pub share: Option<CKReferencePayload>,
    pub encryption_scope: Option<i32>,
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
pub(crate) struct CKNotificationInfoPayload {
    pub alert_body: Option<String>,
    pub alert_localization_key: Option<String>,
    pub alert_localization_args: Option<Vec<String>>,
    pub title: Option<String>,
    pub title_localization_key: Option<String>,
    pub title_localization_args: Option<Vec<String>>,
    pub subtitle: Option<String>,
    pub subtitle_localization_key: Option<String>,
    pub subtitle_localization_args: Option<Vec<String>>,
    pub alert_action_localization_key: Option<String>,
    pub alert_launch_image: Option<String>,
    pub sound_name: Option<String>,
    pub desired_keys: Option<Vec<String>>,
    pub should_badge: bool,
    pub should_send_content_available: bool,
    pub should_send_mutable_content: bool,
    pub category: Option<String>,
    #[serde(rename = "collapseIDKey")]
    pub collapse_id_key: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum CKSubscriptionPayloadKind {
    Query,
    RecordZone,
    Database,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKSubscriptionPayload {
    pub kind: CKSubscriptionPayloadKind,
    #[serde(rename = "subscriptionID")]
    pub subscription_id: String,
    #[serde(rename = "subscriptionType")]
    pub subscription_type: i32,
    #[serde(rename = "notificationInfo")]
    pub notification_info: Option<CKNotificationInfoPayload>,
    pub record_type: Option<String>,
    pub predicate_format: Option<String>,
    #[serde(rename = "zoneID")]
    pub zone_id: Option<CKRecordZoneIDPayload>,
    #[serde(rename = "querySubscriptionOptions")]
    pub query_subscription_options: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKPersonNameComponentsPayload {
    pub name_prefix: Option<String>,
    pub given_name: Option<String>,
    pub middle_name: Option<String>,
    pub family_name: Option<String>,
    pub name_suffix: Option<String>,
    pub nickname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKUserIdentityLookupInfoPayload {
    pub email_address: Option<String>,
    pub phone_number: Option<String>,
    #[serde(rename = "userRecordID")]
    pub user_record_id: Option<CKRecordIDPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKUserIdentityPayload {
    pub archived_data: Vec<u8>,
    #[serde(rename = "userRecordID")]
    pub user_record_id: Option<CKRecordIDPayload>,
    pub lookup_info: Option<CKUserIdentityLookupInfoPayload>,
    pub name_components: Option<CKPersonNameComponentsPayload>,
    pub hasi_cloud_account: bool,
    pub contact_identifiers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKShareParticipantPayload {
    pub archived_data: Vec<u8>,
    pub user_identity: CKUserIdentityPayload,
    pub role: Option<i32>,
    pub permission: i32,
    pub acceptance_status: i32,
    #[serde(rename = "participantID")]
    pub participant_id: String,
    #[serde(rename = "isApprovedRequester")]
    pub is_approved_requester: Option<bool>,
    #[serde(rename = "dateAddedToShare")]
    pub date_added_to_share: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKSharePayload {
    #[serde(rename = "shareRecord")]
    pub share_record: CKRecordPayload,
    #[serde(rename = "rootRecord")]
    pub root_record: Option<CKRecordPayload>,
    #[serde(rename = "zoneID")]
    pub zone_id: CKRecordZoneIDPayload,
    pub public_permission: i32,
    pub url: Option<String>,
    pub participants: Vec<CKShareParticipantPayload>,
    pub owner: Option<CKShareParticipantPayload>,
    #[serde(rename = "currentUserParticipant")]
    pub current_user_participant: Option<CKShareParticipantPayload>,
    pub title: Option<String>,
    #[serde(rename = "thumbnailImageData")]
    pub thumbnail_image_data: Option<Vec<u8>>,
    #[serde(rename = "shareType")]
    pub share_type: Option<String>,
    #[serde(rename = "allowsAccessRequests")]
    pub allows_access_requests: Option<bool>,
    #[serde(rename = "isZoneWide")]
    pub is_zone_wide: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKServerChangeTokenPayload {
    pub archived_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKQueryCursorPayload {
    pub archived_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKFetchedQueryResultsPayload {
    pub records: Vec<CKRecordPayload>,
    pub matches: Vec<CKQueryMatchResultPayload>,
    pub cursor: Option<CKQueryCursorPayload>,
    #[serde(rename = "operationError")]
    pub operation_error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKModifyRecordsOperationPayload {
    pub records_to_save: Vec<CKRecordPayload>,
    #[serde(rename = "recordIDsToDelete")]
    pub record_ids_to_delete: Vec<CKRecordIDPayload>,
    pub save_policy: i32,
    pub atomic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKRecordSaveResultPayload {
    #[serde(rename = "recordID")]
    pub record_id: CKRecordIDPayload,
    pub record: Option<CKRecordPayload>,
    pub error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKRecordDeleteResultPayload {
    #[serde(rename = "recordID")]
    pub record_id: CKRecordIDPayload,
    pub error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKModifyRecordsResultPayload {
    pub saved_records: Vec<CKRecordPayload>,
    #[serde(rename = "deletedRecordIDs")]
    pub deleted_record_ids: Vec<CKRecordIDPayload>,
    #[serde(rename = "saveResults")]
    pub save_results: Vec<CKRecordSaveResultPayload>,
    #[serde(rename = "deleteResults")]
    pub delete_results: Vec<CKRecordDeleteResultPayload>,
    #[serde(rename = "operationError")]
    pub operation_error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKQueryOperationPayload {
    pub query: CKQueryPayload,
    #[serde(rename = "zoneID")]
    pub zone_id: Option<CKRecordZoneIDPayload>,
    pub desired_keys: Option<Vec<String>>,
    pub results_limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKQueryMatchResultPayload {
    #[serde(rename = "recordID")]
    pub record_id: CKRecordIDPayload,
    pub record: Option<CKRecordPayload>,
    pub error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKQueryOperationResultPayload {
    pub records: Vec<CKRecordPayload>,
    pub matches: Vec<CKQueryMatchResultPayload>,
    #[serde(rename = "cursorReturned")]
    pub cursor_returned: bool,
    #[serde(rename = "operationError")]
    pub operation_error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKFetchRecordsOperationPayload {
    #[serde(rename = "recordIDs")]
    pub record_ids: Vec<CKRecordIDPayload>,
    pub desired_keys: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKRecordResultPayload {
    #[serde(rename = "recordID")]
    pub record_id: CKRecordIDPayload,
    pub record: Option<CKRecordPayload>,
    pub error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKFetchRecordsResultPayload {
    pub records: Vec<CKRecordPayload>,
    pub results: Vec<CKRecordResultPayload>,
    #[serde(rename = "operationError")]
    pub operation_error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKFetchDatabaseChangesOperationPayload {
    #[serde(rename = "previousServerChangeToken")]
    pub previous_server_change_token: Option<CKServerChangeTokenPayload>,
    pub results_limit: Option<usize>,
    #[serde(rename = "fetchAllChanges")]
    pub fetch_all_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKFetchDatabaseChangesResultPayload {
    #[serde(rename = "changedZoneIDs")]
    pub changed_zone_ids: Vec<CKRecordZoneIDPayload>,
    #[serde(rename = "deletedZoneIDs")]
    pub deleted_zone_ids: Vec<CKRecordZoneIDPayload>,
    #[serde(rename = "purgedZoneIDs")]
    pub purged_zone_ids: Vec<CKRecordZoneIDPayload>,
    #[serde(rename = "encryptedDataResetZoneIDs")]
    pub encrypted_data_reset_zone_ids: Vec<CKRecordZoneIDPayload>,
    #[serde(rename = "updatedServerChangeTokens")]
    pub updated_server_change_tokens: Vec<CKServerChangeTokenPayload>,
    #[serde(rename = "serverChangeToken")]
    pub server_change_token: Option<CKServerChangeTokenPayload>,
    #[serde(rename = "moreComing")]
    pub more_coming: bool,
    #[serde(rename = "operationError")]
    pub operation_error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKFetchRecordZoneChangesConfigurationPayload {
    #[serde(rename = "previousServerChangeToken")]
    pub previous_server_change_token: Option<CKServerChangeTokenPayload>,
    pub results_limit: Option<usize>,
    pub desired_keys: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKFetchRecordZoneChangesConfigurationEntryPayload {
    #[serde(rename = "zoneID")]
    pub zone_id: CKRecordZoneIDPayload,
    pub configuration: CKFetchRecordZoneChangesConfigurationPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKDeletedRecordPayload {
    #[serde(rename = "recordID")]
    pub record_id: CKRecordIDPayload,
    pub record_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKFetchRecordZoneResultPayload {
    #[serde(rename = "zoneID")]
    pub zone_id: CKRecordZoneIDPayload,
    pub changed_records: Vec<CKRecordPayload>,
    pub deleted_records: Vec<CKDeletedRecordPayload>,
    #[serde(rename = "updatedServerChangeTokens")]
    pub updated_server_change_tokens: Vec<CKServerChangeTokenPayload>,
    #[serde(rename = "serverChangeToken")]
    pub server_change_token: Option<CKServerChangeTokenPayload>,
    #[serde(rename = "clientChangeTokenData")]
    pub client_change_token_data: Option<Vec<u8>>,
    #[serde(rename = "moreComing")]
    pub more_coming: bool,
    #[serde(rename = "zoneError")]
    pub zone_error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKFetchRecordZoneChangesOperationPayload {
    pub zones: Vec<CKFetchRecordZoneChangesConfigurationEntryPayload>,
    #[serde(rename = "fetchAllChanges")]
    pub fetch_all_changes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKFetchRecordZoneChangesResultPayload {
    pub zones: Vec<CKFetchRecordZoneResultPayload>,
    #[serde(rename = "operationError")]
    pub operation_error: Option<ErrorPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKShareMetadataPayload {
    pub archived_data: Vec<u8>,
    pub container_identifier: String,
    pub share: CKSharePayload,
    #[serde(rename = "hierarchicalRootRecordID")]
    pub hierarchical_root_record_id: Option<CKRecordIDPayload>,
    pub participant_role: Option<i32>,
    pub participant_status: i32,
    pub participant_permission: i32,
    pub owner_identity: CKUserIdentityPayload,
    pub root_record: Option<CKRecordPayload>,
    pub participant_type: Option<i32>,
    #[serde(rename = "rootRecordID")]
    pub root_record_id: Option<CKRecordIDPayload>,
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
