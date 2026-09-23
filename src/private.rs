use core::ffi::{c_char, c_void};
use std::collections::BTreeMap;
use std::ffi::{CStr, CString};
use std::time::Duration;

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
    #[serde(rename = "fileURL")]
    pub file_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKLocationPayload {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub horizontal_accuracy: f64,
    pub vertical_accuracy: f64,
    pub course: f64,
    pub speed: f64,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum CKRecordValuePayload {
    String {
        #[serde(rename = "stringValue")]
        value: String,
    },
    Int {
        #[serde(rename = "intValue")]
        value: i64,
    },
    Double {
        #[serde(rename = "doubleValue")]
        value: f64,
    },
    Bool {
        #[serde(rename = "boolValue")]
        value: bool,
    },
    Bytes {
        #[serde(rename = "bytesValue")]
        value: Vec<u8>,
    },
    Date {
        #[serde(rename = "dateValue")]
        value: String,
    },
    Asset {
        #[serde(rename = "assetValue")]
        value: CKAssetPayload,
    },
    Reference {
        #[serde(rename = "referenceValue")]
        value: CKReferencePayload,
    },
    Location {
        #[serde(rename = "locationValue")]
        value: CKLocationPayload,
    },
    Array {
        #[serde(rename = "arrayValue")]
        value: Vec<CKRecordValuePayload>,
    },
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
    pub relative_location: Option<CKLocationPayload>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ComparisonOperatorPayload {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum PredicatePayload {
    TruePredicate,
    FalsePredicate,
    Comparison {
        key: String,
        comparison: ComparisonOperatorPayload,
        value: CKRecordValuePayload,
    },
    BeginsWith {
        key: String,
        text: String,
    },
    IsIn {
        key: String,
        values: Vec<CKRecordValuePayload>,
    },
    NotIn {
        key: String,
        values: Vec<CKRecordValuePayload>,
    },
    ListContains {
        key: String,
        value: CKRecordValuePayload,
    },
    Between {
        key: String,
        values: Vec<CKRecordValuePayload>,
    },
    ContainsTokens {
        text: String,
    },
    WithinDistance {
        key: String,
        location: CKLocationPayload,
        #[serde(rename = "radiusKilometers")]
        radius_kilometers: f64,
    },
    And {
        subpredicates: Vec<PredicatePayload>,
    },
    Not {
        subpredicates: Vec<PredicatePayload>,
    },
    Format {
        format: String,
        arguments: Vec<CKRecordValuePayload>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CKQueryPayload {
    pub record_type: String,
    pub predicate: PredicatePayload,
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
    pub predicate: Option<PredicatePayload>,
    pub predicate_format: Option<String>,
    #[serde(rename = "zoneID")]
    pub zone_id: Option<CKRecordZoneIDPayload>,
    #[serde(rename = "querySubscriptionOptions")]
    pub query_subscription_options: Option<u64>,
    pub archived_data: Option<Vec<u8>>,
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
    pub cursor: Option<CKQueryCursorPayload>,
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
    pub cursor: Option<CKQueryCursorPayload>,
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

pub(crate) fn timeout_seconds(timeout: Option<Duration>) -> f64 {
    timeout.map_or(-1.0, |timeout| timeout.as_secs_f64())
}

pub(crate) fn opt_cstring_ptr(value: &Option<CString>) -> *const c_char {
    value
        .as_ref()
        .map_or(core::ptr::null(), |value| value.as_c_str().as_ptr())
}

/// # Safety
/// `ptr` must be either null or a valid pointer to a bridge-allocated null-terminated C string; this function frees the string via `ck_string_free`.
pub(crate) unsafe fn take_string(ptr: *mut c_char) -> Option<String> {
    doom_fish_utils::ffi_string::take_owned_cstring_c(ptr, |p| ffi::ck_string_free(p))
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

/// # Safety
/// `ptr` must be either null or a valid pointer to a bridge-allocated null-terminated C string; this function frees the string via `ck_string_free`.
pub(crate) unsafe fn parse_json_ptr<T: DeserializeOwned>(
    ptr: *mut c_char,
    context: &str,
) -> Result<T, CloudKitError> {
    let json = take_string(ptr)
        .ok_or_else(|| CloudKitError::bridge(-1, format!("missing JSON payload for {context}")))?;
    parse_json_str(&json, context)
}

/// # Safety
/// `ptr` must be either null or a valid pointer to a bridge-allocated null-terminated C string; this function frees the string via `ck_string_free`.
pub(crate) unsafe fn parse_error_ptr(ptr: *mut c_char) -> CloudKitError {
    if ptr.is_null() {
        return CloudKitError::bridge(-2, "CloudKit bridge returned an error without payload");
    }
    let json = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    ffi::ck_string_free(ptr);
    parse_error_json_str(&json)
}

/// # Safety
/// `ptr` must be null or point to a valid null-terminated C string for the duration of this call; this function borrows the string and does not free it.
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

/// # Safety
/// `err_msg` must be either null or a valid pointer to a bridge-allocated null-terminated C string; when non-null it is consumed via `parse_error_ptr`.
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::record::{CKAsset, RecordLocation, RecordValue};

    #[test]
    fn record_values_without_their_payload_are_rejected() {
        for value in [
            json!({"kind": "string"}),
            json!({"kind": "int", "stringValue": "5"}),
            json!({"kind": "location", "locationValue": {"latitude": 1.0}}),
            json!({"kind": "unknownKind", "stringValue": "x"}),
        ] {
            assert!(
                serde_json::from_value::<CKRecordValuePayload>(value.clone()).is_err(),
                "{value}"
            );
        }
    }

    #[test]
    fn record_values_decode_the_swift_wire_format() {
        let asset: CKRecordValuePayload =
            serde_json::from_value(json!({"kind": "asset", "assetValue": {"fileURL": "/tmp/a"}}))
                .expect("Swift encodes CKAsset.fileURL under fileURL");
        assert_eq!(
            RecordValue::from_payload(asset),
            RecordValue::Asset(CKAsset::new("/tmp/a"))
        );

        let missing_file: CKRecordValuePayload =
            serde_json::from_value(json!({"kind": "asset", "assetValue": {}}))
                .expect("CKAsset.fileURL may be nil");
        assert!(matches!(
            RecordValue::from_payload(missing_file),
            RecordValue::Asset(asset) if asset.file_url().is_none()
        ));

        let location: CKRecordValuePayload = serde_json::from_value(json!({
            "kind": "location",
            "locationValue": {
                "latitude": 59.3,
                "longitude": 18.0,
                "altitude": 12.0,
                "horizontalAccuracy": 5.0,
                "verticalAccuracy": 3.0,
                "course": 90.0,
                "speed": 1.5,
                "timestamp": "2023-11-14T22:13:20.000Z"
            }
        }))
        .expect("Swift encodes CLLocation values");
        assert_eq!(
            RecordValue::from_payload(location),
            RecordValue::Location(RecordLocation {
                latitude: 59.3,
                longitude: 18.0,
                altitude: 12.0,
                horizontal_accuracy: 5.0,
                vertical_accuracy: 3.0,
                course: 90.0,
                speed: 1.5,
                timestamp: Some("2023-11-14T22:13:20.000Z".into()),
            })
        );
    }

    #[test]
    fn record_values_encode_the_swift_wire_format() {
        let encoded = serde_json::to_value(
            RecordValue::Array(vec![
                RecordValue::Asset(CKAsset::new("/tmp/a")),
                RecordValue::Location(RecordLocation::new(1.0, 2.0)),
                RecordValue::Int(i64::MAX),
            ])
            .to_payload(),
        )
        .expect("record values serialize");

        assert_eq!(encoded["kind"], "array");
        assert_eq!(encoded["arrayValue"][0]["assetValue"]["fileURL"], "/tmp/a");
        assert_eq!(encoded["arrayValue"][1]["kind"], "location");
        assert_eq!(
            encoded["arrayValue"][1]["locationValue"]["verticalAccuracy"],
            -1.0
        );
        assert_eq!(encoded["arrayValue"][2]["intValue"], i64::MAX);
    }

    #[test]
    fn record_values_round_trip_every_kind() {
        let value = RecordValue::Array(vec![
            RecordValue::String("s".into()),
            RecordValue::Int(-3),
            RecordValue::Double(0.25),
            RecordValue::Bool(false),
            RecordValue::Bytes(vec![0, 255]),
            RecordValue::Date("2024-01-01T00:00:00.000Z".into()),
            RecordValue::Asset(CKAsset::new("/tmp/b")),
            RecordValue::Location(RecordLocation::new(-33.9, 151.2)),
        ]);
        let json = serde_json::to_string(&value.to_payload()).expect("serialize");
        let decoded: CKRecordValuePayload = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(RecordValue::from_payload(decoded), value);
    }

    #[test]
    fn timeouts_map_to_bridge_seconds() {
        assert!((timeout_seconds(None) + 1.0).abs() < f64::EPSILON);
        assert!((timeout_seconds(Some(Duration::from_millis(1500))) - 1.5).abs() < f64::EPSILON);
    }
}
