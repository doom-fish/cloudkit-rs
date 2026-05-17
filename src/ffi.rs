#![allow(missing_docs, non_camel_case_types)]

use core::ffi::{c_char, c_void};

extern "C" {
    pub fn ck_string_free(s: *mut c_char);

    pub fn ck_record_create(
        record_type: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;

    pub fn ck_container_account_status_sync(
        container_identifier: *const c_char,
        out_status: *mut i32,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_container_account_status_async(
        container_identifier: *const c_char,
        callback: AccountStatusCallback,
        refcon: *mut c_void,
    );

    pub fn ck_container_fetch_user_record_id_sync(
        container_identifier: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_container_fetch_user_record_id_async(
        container_identifier: *const c_char,
        callback: JsonCallback,
        refcon: *mut c_void,
    );

    pub fn ck_container_discover_user_identity_sync(
        container_identifier: *const c_char,
        lookup_info_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_container_fetch_share_participant_sync(
        container_identifier: *const c_char,
        lookup_info_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;

    pub fn ck_database_save_record_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        record_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_fetch_record_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        record_id_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_delete_record_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        record_id_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_perform_query_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        query_json: *const c_char,
        zone_id_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_perform_query_async(
        container_identifier: *const c_char,
        database_scope: i32,
        query_json: *const c_char,
        zone_id_json: *const c_char,
        callback: JsonCallback,
        refcon: *mut c_void,
    );
    pub fn ck_database_fetch_query_results_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        query_json: *const c_char,
        zone_id_json: *const c_char,
        desired_keys_json: *const c_char,
        results_limit: i32,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;

    pub fn ck_database_fetch_all_record_zones_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_fetch_record_zone_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        zone_id_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_save_record_zone_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        zone_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_delete_record_zone_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        zone_id_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;

    pub fn ck_database_fetch_subscription_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        subscription_id: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_fetch_all_subscriptions_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_save_subscription_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        subscription_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_delete_subscription_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        subscription_id: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;

    pub fn ck_database_execute_modify_records_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        operation_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_execute_query_operation_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        operation_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_execute_fetch_records_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        operation_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_execute_fetch_database_changes_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        operation_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_execute_fetch_record_zone_changes_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        operation_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;

    pub fn ck_share_create_root_record(
        root_record_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_share_create_zone_wide(
        zone_id_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_share_normalize(
        share_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_share_create_one_time_url_participant(
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;

    pub fn ck_container_fetch_share_metadata_sync(
        container_identifier: *const c_char,
        share_url: *const c_char,
        should_fetch_root_record: bool,
        root_record_desired_keys_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_container_accept_share_metadata_sync(
        container_identifier: *const c_char,
        share_metadata_json: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_container_request_share_access_sync(
        container_identifier: *const c_char,
        share_url: *const c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
    pub fn ck_database_execute_fetch_web_auth_token_sync(
        container_identifier: *const c_char,
        database_scope: i32,
        api_token: *const c_char,
        out_json: *mut *mut c_char,
        out_error_json: *mut *mut c_char,
    ) -> i32;
}

pub type JsonCallback =
    unsafe extern "C" fn(refcon: *mut c_void, json: *const c_char, error_json: *const c_char);
pub type AccountStatusCallback =
    unsafe extern "C" fn(refcon: *mut c_void, status_raw: i32, error_json: *const c_char);

pub mod status {
    pub const OK: i32 = 0;
    pub const INVALID_ARGUMENT: i32 = -1;
    pub const FAILURE: i32 = -2;
    pub const TIMED_OUT: i32 = -3;
}
