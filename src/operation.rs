use core::ffi::c_char;
use core::ptr;

use crate::database::CKDatabase;
use crate::error::CloudKitError;
use crate::ffi;
use crate::private::{
    error_from_status, json_cstring, opt_cstring_ptr, optional_cstring_from_str, parse_json_ptr,
    CKModifyRecordsOperationPayload, CKModifyRecordsResultPayload, CKQueryOperationPayload,
    CKQueryOperationResultPayload,
};
use crate::query::CKQuery;
use crate::record::{CKRecord, CKRecordID, CKRecordZoneID};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CKRecordSavePolicy {
    IfServerRecordUnchanged = 0,
    ChangedKeys = 1,
    AllKeys = 2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKRecordSaveResult {
    pub record_id: CKRecordID,
    pub record: Option<CKRecord>,
    pub error: Option<crate::error::CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKRecordDeleteResult {
    pub record_id: CKRecordID,
    pub error: Option<crate::error::CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModifyRecordsResult {
    pub saved_records: Vec<CKRecord>,
    pub deleted_record_ids: Vec<CKRecordID>,
    pub save_results: Vec<CKRecordSaveResult>,
    pub delete_results: Vec<CKRecordDeleteResult>,
    pub operation_error: Option<crate::error::CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryMatchResult {
    pub record_id: CKRecordID,
    pub record: Option<CKRecord>,
    pub error: Option<crate::error::CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryOperationResult {
    pub records: Vec<CKRecord>,
    pub matches: Vec<QueryMatchResult>,
    pub cursor_returned: bool,
    pub operation_error: Option<crate::error::CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKModifyRecordsOperation {
    records_to_save: Vec<CKRecord>,
    record_ids_to_delete: Vec<CKRecordID>,
    save_policy: CKRecordSavePolicy,
    atomic: bool,
}

impl CKModifyRecordsOperation {
    pub fn new(records_to_save: Vec<CKRecord>, record_ids_to_delete: Vec<CKRecordID>) -> Self {
        Self {
            records_to_save,
            record_ids_to_delete,
            save_policy: CKRecordSavePolicy::IfServerRecordUnchanged,
            atomic: true,
        }
    }

    pub fn records_to_save(&self) -> &[CKRecord] {
        &self.records_to_save
    }

    pub fn record_ids_to_delete(&self) -> &[CKRecordID] {
        &self.record_ids_to_delete
    }

    pub const fn save_policy(&self) -> CKRecordSavePolicy {
        self.save_policy
    }

    pub const fn atomic(&self) -> bool {
        self.atomic
    }

    pub fn with_save_policy(mut self, save_policy: CKRecordSavePolicy) -> Self {
        self.save_policy = save_policy;
        self
    }

    pub fn with_atomic(mut self, atomic: bool) -> Self {
        self.atomic = atomic;
        self
    }

    pub fn execute_in(&self, database: &CKDatabase) -> Result<ModifyRecordsResult, CloudKitError> {
        let identifier = optional_cstring_from_str(
            database.container().container_identifier(),
            "container identifier",
        )?;
        let payload = CKModifyRecordsOperationPayload {
            records_to_save: self
                .records_to_save
                .iter()
                .map(CKRecord::to_payload)
                .collect(),
            record_ids_to_delete: self
                .record_ids_to_delete
                .iter()
                .map(CKRecordID::to_payload)
                .collect(),
            save_policy: self.save_policy as i32,
            atomic: self.atomic,
        };
        let operation_json = json_cstring(&payload, "modify-records operation")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::ck_database_execute_modify_records_sync(
                opt_cstring_ptr(&identifier),
                database.database_scope() as i32,
                operation_json.as_ptr(),
                &mut out_json,
                &mut out_error,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        let payload = unsafe {
            parse_json_ptr::<CKModifyRecordsResultPayload>(out_json, "modify-records result")?
        };
        Ok(ModifyRecordsResult {
            saved_records: payload
                .saved_records
                .into_iter()
                .map(CKRecord::from_payload)
                .collect(),
            deleted_record_ids: payload
                .deleted_record_ids
                .into_iter()
                .map(CKRecordID::from_payload)
                .collect(),
            save_results: payload
                .save_results
                .into_iter()
                .map(|entry| CKRecordSaveResult {
                    record_id: CKRecordID::from_payload(entry.record_id),
                    record: entry.record.map(CKRecord::from_payload),
                    error: entry.error.map(crate::error::CloudKitError::from_payload),
                })
                .collect(),
            delete_results: payload
                .delete_results
                .into_iter()
                .map(|entry| CKRecordDeleteResult {
                    record_id: CKRecordID::from_payload(entry.record_id),
                    error: entry.error.map(crate::error::CloudKitError::from_payload),
                })
                .collect(),
            operation_error: payload
                .operation_error
                .map(crate::error::CloudKitError::from_payload),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKQueryOperation {
    query: CKQuery,
    zone_id: Option<CKRecordZoneID>,
    desired_keys: Option<Vec<String>>,
    results_limit: Option<usize>,
}

impl CKQueryOperation {
    pub fn new(query: CKQuery) -> Self {
        Self {
            query,
            zone_id: None,
            desired_keys: None,
            results_limit: None,
        }
    }

    pub fn query(&self) -> &CKQuery {
        &self.query
    }

    pub const fn zone_id(&self) -> Option<&CKRecordZoneID> {
        self.zone_id.as_ref()
    }

    pub fn desired_keys(&self) -> Option<&[String]> {
        self.desired_keys.as_deref()
    }

    pub const fn results_limit(&self) -> Option<usize> {
        self.results_limit
    }

    pub fn with_zone_id(mut self, zone_id: CKRecordZoneID) -> Self {
        self.zone_id = Some(zone_id);
        self
    }

    pub fn with_desired_keys(mut self, desired_keys: Vec<String>) -> Self {
        self.desired_keys = Some(desired_keys);
        self
    }

    pub fn with_results_limit(mut self, results_limit: usize) -> Self {
        self.results_limit = Some(results_limit);
        self
    }

    pub fn execute_in(&self, database: &CKDatabase) -> Result<QueryOperationResult, CloudKitError> {
        let identifier = optional_cstring_from_str(
            database.container().container_identifier(),
            "container identifier",
        )?;
        let payload = CKQueryOperationPayload {
            query: self.query.to_payload(),
            zone_id: self.zone_id.as_ref().map(CKRecordZoneID::to_payload),
            desired_keys: self.desired_keys.clone(),
            results_limit: self.results_limit,
        };
        let operation_json = json_cstring(&payload, "query operation")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::ck_database_execute_query_operation_sync(
                opt_cstring_ptr(&identifier),
                database.database_scope() as i32,
                operation_json.as_ptr(),
                &mut out_json,
                &mut out_error,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        let payload = unsafe {
            parse_json_ptr::<CKQueryOperationResultPayload>(out_json, "query operation result")?
        };
        Ok(QueryOperationResult {
            records: payload
                .records
                .into_iter()
                .map(CKRecord::from_payload)
                .collect(),
            matches: payload
                .matches
                .into_iter()
                .map(|entry| QueryMatchResult {
                    record_id: CKRecordID::from_payload(entry.record_id),
                    record: entry.record.map(CKRecord::from_payload),
                    error: entry.error.map(crate::error::CloudKitError::from_payload),
                })
                .collect(),
            cursor_returned: payload.cursor_returned,
            operation_error: payload
                .operation_error
                .map(crate::error::CloudKitError::from_payload),
        })
    }
}
