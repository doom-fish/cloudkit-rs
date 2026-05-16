use core::ffi::{c_char, c_void};
use core::ptr;

use crate::container::CKContainer;
use crate::error::CloudKitError;
use crate::ffi;
use crate::private::{
    box_closure, error_from_status, json_cstring, opt_cstring_ptr, optional_cstring_from_str,
    parse_borrowed_error_ptr, parse_json_ptr, parse_json_str,
};
use crate::query::CKQuery;
use crate::record::{CKRecord, CKRecordID, CKRecordZoneID};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CKDatabaseScope {
    Public = 1,
    Private = 2,
    Shared = 3,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKDatabase {
    container: CKContainer,
    database_scope: CKDatabaseScope,
}

impl CKDatabase {
    pub(crate) const fn new(container: CKContainer, database_scope: CKDatabaseScope) -> Self {
        Self {
            container,
            database_scope,
        }
    }

    pub const fn database_scope(&self) -> CKDatabaseScope {
        self.database_scope
    }

    pub const fn container(&self) -> &CKContainer {
        &self.container
    }

    pub fn save_record(&self, record: &CKRecord) -> Result<CKRecord, CloudKitError> {
        let identifier = optional_cstring_from_str(
            self.container.container_identifier(),
            "container identifier",
        )?;
        let record_json = json_cstring(&record.to_payload(), "record")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::ck_database_save_record_sync(
                opt_cstring_ptr(&identifier),
                self.database_scope as i32,
                record_json.as_ptr(),
                &mut out_json,
                &mut out_error,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        let payload =
            unsafe { parse_json_ptr::<crate::private::CKRecordPayload>(out_json, "saved record")? };
        Ok(CKRecord::from_payload(payload))
    }

    pub fn fetch_record(&self, record_id: &CKRecordID) -> Result<CKRecord, CloudKitError> {
        let identifier = optional_cstring_from_str(
            self.container.container_identifier(),
            "container identifier",
        )?;
        let record_id_json = json_cstring(&record_id.to_payload(), "record ID")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::ck_database_fetch_record_sync(
                opt_cstring_ptr(&identifier),
                self.database_scope as i32,
                record_id_json.as_ptr(),
                &mut out_json,
                &mut out_error,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        let payload = unsafe {
            parse_json_ptr::<crate::private::CKRecordPayload>(out_json, "fetched record")?
        };
        Ok(CKRecord::from_payload(payload))
    }

    pub fn delete_record(&self, record_id: &CKRecordID) -> Result<CKRecordID, CloudKitError> {
        let identifier = optional_cstring_from_str(
            self.container.container_identifier(),
            "container identifier",
        )?;
        let record_id_json = json_cstring(&record_id.to_payload(), "record ID")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::ck_database_delete_record_sync(
                opt_cstring_ptr(&identifier),
                self.database_scope as i32,
                record_id_json.as_ptr(),
                &mut out_json,
                &mut out_error,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        let payload = unsafe {
            parse_json_ptr::<crate::private::CKRecordIDPayload>(out_json, "deleted record ID")?
        };
        Ok(CKRecordID::from_payload(payload))
    }

    pub fn perform_query(
        &self,
        query: &CKQuery,
        zone_id: Option<&CKRecordZoneID>,
    ) -> Result<Vec<CKRecord>, CloudKitError> {
        let identifier = optional_cstring_from_str(
            self.container.container_identifier(),
            "container identifier",
        )?;
        let query_json = json_cstring(&query.to_payload(), "query")?;
        let zone_json = zone_id.map(CKRecordZoneID::to_payload);
        let zone_json = match zone_json.as_ref() {
            Some(zone) => Some(json_cstring(zone, "zone ID")?),
            None => None,
        };
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::ck_database_perform_query_sync(
                opt_cstring_ptr(&identifier),
                self.database_scope as i32,
                query_json.as_ptr(),
                opt_cstring_ptr(&zone_json),
                &mut out_json,
                &mut out_error,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        let payloads = unsafe {
            parse_json_ptr::<Vec<crate::private::CKRecordPayload>>(out_json, "query results")?
        };
        Ok(payloads.into_iter().map(CKRecord::from_payload).collect())
    }

    pub fn perform_query_with_completion_handler<F>(
        &self,
        query: &CKQuery,
        zone_id: Option<&CKRecordZoneID>,
        callback: F,
    ) -> Result<(), CloudKitError>
    where
        F: FnOnce(Result<Vec<CKRecord>, CloudKitError>) + Send + 'static,
    {
        let identifier = optional_cstring_from_str(
            self.container.container_identifier(),
            "container identifier",
        )?;
        let query_json = json_cstring(&query.to_payload(), "query")?;
        let zone_json = zone_id.map(CKRecordZoneID::to_payload);
        let zone_json = match zone_json.as_ref() {
            Some(zone) => Some(json_cstring(zone, "zone ID")?),
            None => None,
        };
        let callback_ptr = box_closure(Box::new(callback) as QueryCallback);
        unsafe {
            ffi::ck_database_perform_query_async(
                opt_cstring_ptr(&identifier),
                self.database_scope as i32,
                query_json.as_ptr(),
                opt_cstring_ptr(&zone_json),
                query_callback_trampoline,
                callback_ptr,
            );
        }
        Ok(())
    }
}

type QueryCallback = Box<dyn FnOnce(Result<Vec<CKRecord>, CloudKitError>) + Send + 'static>;

unsafe extern "C" fn query_callback_trampoline(
    refcon: *mut c_void,
    json: *const c_char,
    error_json: *const c_char,
) {
    let callback: Box<QueryCallback> = Box::from_raw(refcon.cast());
    let result = if error_json.is_null() {
        let payloads = parse_json_str::<Vec<crate::private::CKRecordPayload>>(
            &std::ffi::CStr::from_ptr(json).to_string_lossy(),
            "query results",
        );
        payloads.map(|payloads| payloads.into_iter().map(CKRecord::from_payload).collect())
    } else {
        Err(parse_borrowed_error_ptr(error_json))
    };
    callback(result);
}
