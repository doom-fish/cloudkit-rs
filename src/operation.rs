use core::ffi::c_char;
use core::ptr;

use crate::database::CKDatabase;
use crate::error::CloudKitError;
use crate::fetched_results::{
    CKFetchDatabaseChangesResult, CKFetchRecordZoneChangesResult, CKFetchRecordsResult,
};
use crate::ffi;
use crate::private::{
    error_from_status, json_cstring, opt_cstring_ptr, optional_cstring_from_str, parse_json_ptr,
    CKFetchDatabaseChangesOperationPayload, CKFetchRecordZoneChangesConfigurationEntryPayload,
    CKFetchRecordZoneChangesConfigurationPayload, CKFetchRecordZoneChangesOperationPayload,
    CKFetchRecordZoneChangesResultPayload, CKFetchRecordsOperationPayload,
    CKFetchRecordsResultPayload, CKModifyRecordsOperationPayload, CKModifyRecordsResultPayload,
    CKQueryOperationPayload, CKQueryOperationResultPayload,
};
use crate::query::CKQuery;
use crate::record::{CKRecord, CKRecordID, CKRecordZoneID};
use crate::server_change_token::CKServerChangeToken;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKFetchRecordsOperation {
    record_ids: Vec<CKRecordID>,
    desired_keys: Option<Vec<String>>,
}

impl CKFetchRecordsOperation {
    pub fn new(record_ids: Vec<CKRecordID>) -> Self {
        Self {
            record_ids,
            desired_keys: None,
        }
    }

    pub fn record_ids(&self) -> &[CKRecordID] {
        &self.record_ids
    }

    pub fn desired_keys(&self) -> Option<&[String]> {
        self.desired_keys.as_deref()
    }

    pub fn with_desired_keys(mut self, desired_keys: Vec<String>) -> Self {
        self.desired_keys = Some(desired_keys);
        self
    }

    pub fn execute_in(&self, database: &CKDatabase) -> Result<CKFetchRecordsResult, CloudKitError> {
        let identifier = optional_cstring_from_str(
            database.container().container_identifier(),
            "container identifier",
        )?;
        let payload = CKFetchRecordsOperationPayload {
            record_ids: self.record_ids.iter().map(CKRecordID::to_payload).collect(),
            desired_keys: self.desired_keys.clone(),
        };
        let operation_json = json_cstring(&payload, "fetch-records operation")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::ck_database_execute_fetch_records_sync(
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
            parse_json_ptr::<CKFetchRecordsResultPayload>(out_json, "fetch-records result")?
        };
        Ok(CKFetchRecordsResult {
            records: payload.records.into_iter().map(CKRecord::from_payload).collect(),
            results: payload
                .results
                .into_iter()
                .map(crate::fetched_results::CKRecordResult::from_payload)
                .collect(),
            operation_error: payload.operation_error.map(CloudKitError::from_payload),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKFetchDatabaseChangesOperation {
    previous_server_change_token: Option<CKServerChangeToken>,
    results_limit: Option<usize>,
    fetch_all_changes: bool,
}

impl CKFetchDatabaseChangesOperation {
    pub fn new() -> Self {
        Self {
            previous_server_change_token: None,
            results_limit: None,
            fetch_all_changes: true,
        }
    }

    pub const fn previous_server_change_token(&self) -> Option<&CKServerChangeToken> {
        self.previous_server_change_token.as_ref()
    }

    pub const fn results_limit(&self) -> Option<usize> {
        self.results_limit
    }

    pub const fn fetch_all_changes(&self) -> bool {
        self.fetch_all_changes
    }

    pub fn with_previous_server_change_token(mut self, token: CKServerChangeToken) -> Self {
        self.previous_server_change_token = Some(token);
        self
    }

    pub fn with_results_limit(mut self, results_limit: usize) -> Self {
        self.results_limit = Some(results_limit);
        self
    }

    pub fn with_fetch_all_changes(mut self, fetch_all_changes: bool) -> Self {
        self.fetch_all_changes = fetch_all_changes;
        self
    }

    pub fn execute_in(&self, database: &CKDatabase) -> Result<CKFetchDatabaseChangesResult, CloudKitError> {
        let identifier = optional_cstring_from_str(
            database.container().container_identifier(),
            "container identifier",
        )?;
        let payload = CKFetchDatabaseChangesOperationPayload {
            previous_server_change_token: self
                .previous_server_change_token
                .as_ref()
                .map(CKServerChangeToken::to_payload),
            results_limit: self.results_limit,
            fetch_all_changes: self.fetch_all_changes,
        };
        let operation_json = json_cstring(&payload, "fetch-database-changes operation")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::ck_database_execute_fetch_database_changes_sync(
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
            parse_json_ptr::<crate::private::CKFetchDatabaseChangesResultPayload>(
                out_json,
                "fetch-database-changes result",
            )?
        };
        Ok(CKFetchDatabaseChangesResult::from_payload(payload))
    }
}

impl Default for CKFetchDatabaseChangesOperation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKFetchRecordZoneChangesConfiguration {
    previous_server_change_token: Option<CKServerChangeToken>,
    results_limit: Option<usize>,
    desired_keys: Option<Vec<String>>,
}

impl CKFetchRecordZoneChangesConfiguration {
    pub fn new() -> Self {
        Self {
            previous_server_change_token: None,
            results_limit: None,
            desired_keys: None,
        }
    }

    pub const fn previous_server_change_token(&self) -> Option<&CKServerChangeToken> {
        self.previous_server_change_token.as_ref()
    }

    pub const fn results_limit(&self) -> Option<usize> {
        self.results_limit
    }

    pub fn desired_keys(&self) -> Option<&[String]> {
        self.desired_keys.as_deref()
    }

    pub fn with_previous_server_change_token(mut self, token: CKServerChangeToken) -> Self {
        self.previous_server_change_token = Some(token);
        self
    }

    pub fn with_results_limit(mut self, results_limit: usize) -> Self {
        self.results_limit = Some(results_limit);
        self
    }

    pub fn with_desired_keys(mut self, desired_keys: Vec<String>) -> Self {
        self.desired_keys = Some(desired_keys);
        self
    }

    pub(crate) fn to_payload(&self) -> CKFetchRecordZoneChangesConfigurationPayload {
        CKFetchRecordZoneChangesConfigurationPayload {
            previous_server_change_token: self
                .previous_server_change_token
                .as_ref()
                .map(CKServerChangeToken::to_payload),
            results_limit: self.results_limit,
            desired_keys: self.desired_keys.clone(),
        }
    }
}

impl Default for CKFetchRecordZoneChangesConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKFetchRecordZoneChangesOperation {
    zones: Vec<(CKRecordZoneID, CKFetchRecordZoneChangesConfiguration)>,
    fetch_all_changes: bool,
}

impl CKFetchRecordZoneChangesOperation {
    pub fn new(record_zone_ids: Vec<CKRecordZoneID>) -> Self {
        Self {
            zones: record_zone_ids
                .into_iter()
                .map(|zone_id| (zone_id, CKFetchRecordZoneChangesConfiguration::default()))
                .collect(),
            fetch_all_changes: true,
        }
    }

    pub fn zones(&self) -> &[(CKRecordZoneID, CKFetchRecordZoneChangesConfiguration)] {
        &self.zones
    }

    pub const fn fetch_all_changes(&self) -> bool {
        self.fetch_all_changes
    }

    pub fn with_zone_configuration(
        mut self,
        zone_id: CKRecordZoneID,
        configuration: CKFetchRecordZoneChangesConfiguration,
    ) -> Self {
        if let Some(existing) = self.zones.iter_mut().find(|(existing_zone_id, _)| *existing_zone_id == zone_id)
        {
            existing.1 = configuration;
        } else {
            self.zones.push((zone_id, configuration));
        }
        self
    }

    pub fn with_fetch_all_changes(mut self, fetch_all_changes: bool) -> Self {
        self.fetch_all_changes = fetch_all_changes;
        self
    }

    pub fn execute_in(
        &self,
        database: &CKDatabase,
    ) -> Result<CKFetchRecordZoneChangesResult, CloudKitError> {
        let identifier = optional_cstring_from_str(
            database.container().container_identifier(),
            "container identifier",
        )?;
        let payload = CKFetchRecordZoneChangesOperationPayload {
            zones: self
                .zones
                .iter()
                .map(|(zone_id, configuration)| CKFetchRecordZoneChangesConfigurationEntryPayload {
                    zone_id: zone_id.to_payload(),
                    configuration: configuration.to_payload(),
                })
                .collect(),
            fetch_all_changes: self.fetch_all_changes,
        };
        let operation_json = json_cstring(&payload, "fetch-record-zone-changes operation")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::ck_database_execute_fetch_record_zone_changes_sync(
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
            parse_json_ptr::<CKFetchRecordZoneChangesResultPayload>(
                out_json,
                "fetch-record-zone-changes result",
            )?
        };
        Ok(CKFetchRecordZoneChangesResult::from_payload(payload))
    }
}
