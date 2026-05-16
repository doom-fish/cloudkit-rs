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
