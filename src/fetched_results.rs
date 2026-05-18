use crate::error::CloudKitError;
use crate::operation::QueryMatchResult;
use crate::private::{
    CKDeletedRecordPayload, CKFetchDatabaseChangesResultPayload,
    CKFetchRecordZoneChangesResultPayload, CKFetchedQueryResultsPayload, CKQueryCursorPayload,
    CKRecordResultPayload,
};
use crate::record::{CKRecord, CKRecordID, CKRecordZoneID};
use crate::server_change_token::CKServerChangeToken;

/// Wraps `CKQueryCursor`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKQueryCursor {
    archived_data: Vec<u8>,
}

impl CKQueryCursor {
    /// Creates this wrapper from archived `CKQueryCursor` data.
    pub fn from_archived_data(archived_data: Vec<u8>) -> Self {
        Self { archived_data }
    }

    /// Mirrors archived data stored by `CKQueryCursor`.
    pub fn archived_data(&self) -> &[u8] {
        &self.archived_data
    }

    pub(crate) fn from_payload(payload: CKQueryCursorPayload) -> Self {
        Self::from_archived_data(payload.archived_data)
    }
}

/// Wraps `CKFetchedQueryResults`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchedQueryResults {
    /// Mirrors `CKFetchedQueryResults.records`.
    pub records: Vec<CKRecord>,
    /// Mirrors `CKFetchedQueryResults.matches`.
    pub matches: Vec<QueryMatchResult>,
    /// Mirrors `CKFetchedQueryResults.cursor`.
    pub cursor: Option<CKQueryCursor>,
    /// Mirrors `CKFetchedQueryResults.operationError`.
    pub operation_error: Option<CloudKitError>,
}

impl CKFetchedQueryResults {
    pub(crate) fn from_payload(payload: CKFetchedQueryResultsPayload) -> Self {
        Self {
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
                    error: entry.error.map(CloudKitError::from_payload),
                })
                .collect(),
            cursor: payload.cursor.map(CKQueryCursor::from_payload),
            operation_error: payload.operation_error.map(CloudKitError::from_payload),
        }
    }
}

/// Wraps `CKRecordResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKRecordResult {
    /// Mirrors `CKRecordResult.recordID`.
    pub record_id: CKRecordID,
    /// Mirrors `CKRecordResult.record`.
    pub record: Option<CKRecord>,
    /// Mirrors `CKRecordResult.error`.
    pub error: Option<CloudKitError>,
}

impl CKRecordResult {
    pub(crate) fn from_payload(payload: CKRecordResultPayload) -> Self {
        Self {
            record_id: CKRecordID::from_payload(payload.record_id),
            record: payload.record.map(CKRecord::from_payload),
            error: payload.error.map(CloudKitError::from_payload),
        }
    }
}

/// Wraps `CKFetchRecordsResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchRecordsResult {
    /// Mirrors `CKFetchRecordsResult.records`.
    pub records: Vec<CKRecord>,
    /// Mirrors `CKFetchRecordsResult.results`.
    pub results: Vec<CKRecordResult>,
    /// Mirrors `CKFetchRecordsResult.operationError`.
    pub operation_error: Option<CloudKitError>,
}

/// Wraps `CKDeletedRecord`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKDeletedRecord {
    /// Mirrors `CKDeletedRecord.recordID`.
    pub record_id: CKRecordID,
    /// Mirrors `CKDeletedRecord.recordType`.
    pub record_type: String,
}

impl CKDeletedRecord {
    pub(crate) fn from_payload(payload: CKDeletedRecordPayload) -> Self {
        Self {
            record_id: CKRecordID::from_payload(payload.record_id),
            record_type: payload.record_type,
        }
    }
}

/// Wraps `CKFetchDatabaseChangesResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchDatabaseChangesResult {
    /// Mirrors `CKFetchDatabaseChangesResult.changedZoneIDs`.
    pub changed_zone_ids: Vec<CKRecordZoneID>,
    /// Mirrors `CKFetchDatabaseChangesResult.deletedZoneIDs`.
    pub deleted_zone_ids: Vec<CKRecordZoneID>,
    /// Mirrors `CKFetchDatabaseChangesResult.purgedZoneIDs`.
    pub purged_zone_ids: Vec<CKRecordZoneID>,
    /// Mirrors `CKFetchDatabaseChangesResult.encryptedDataResetZoneIDs`.
    pub encrypted_data_reset_zone_ids: Vec<CKRecordZoneID>,
    /// Mirrors `CKFetchDatabaseChangesResult.updatedServerChangeTokens`.
    pub updated_server_change_tokens: Vec<CKServerChangeToken>,
    /// Mirrors `CKFetchDatabaseChangesResult.serverChangeToken`.
    pub server_change_token: Option<CKServerChangeToken>,
    /// Mirrors `CKFetchDatabaseChangesResult.moreComing`.
    pub more_coming: bool,
    /// Mirrors `CKFetchDatabaseChangesResult.operationError`.
    pub operation_error: Option<CloudKitError>,
}

impl CKFetchDatabaseChangesResult {
    pub(crate) fn from_payload(payload: CKFetchDatabaseChangesResultPayload) -> Self {
        Self {
            changed_zone_ids: payload
                .changed_zone_ids
                .into_iter()
                .map(CKRecordZoneID::from_payload)
                .collect(),
            deleted_zone_ids: payload
                .deleted_zone_ids
                .into_iter()
                .map(CKRecordZoneID::from_payload)
                .collect(),
            purged_zone_ids: payload
                .purged_zone_ids
                .into_iter()
                .map(CKRecordZoneID::from_payload)
                .collect(),
            encrypted_data_reset_zone_ids: payload
                .encrypted_data_reset_zone_ids
                .into_iter()
                .map(CKRecordZoneID::from_payload)
                .collect(),
            updated_server_change_tokens: payload
                .updated_server_change_tokens
                .into_iter()
                .map(CKServerChangeToken::from_payload)
                .collect(),
            server_change_token: payload
                .server_change_token
                .map(CKServerChangeToken::from_payload),
            more_coming: payload.more_coming,
            operation_error: payload.operation_error.map(CloudKitError::from_payload),
        }
    }
}

/// Wraps `CKFetchRecordZoneResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchRecordZoneResult {
    /// Mirrors `CKFetchRecordZoneResult.zoneID`.
    pub zone_id: CKRecordZoneID,
    /// Mirrors `CKFetchRecordZoneResult.changedRecords`.
    pub changed_records: Vec<CKRecord>,
    /// Mirrors `CKFetchRecordZoneResult.deletedRecords`.
    pub deleted_records: Vec<CKDeletedRecord>,
    /// Mirrors `CKFetchRecordZoneResult.updatedServerChangeTokens`.
    pub updated_server_change_tokens: Vec<CKServerChangeToken>,
    /// Mirrors `CKFetchRecordZoneResult.serverChangeToken`.
    pub server_change_token: Option<CKServerChangeToken>,
    /// Mirrors `CKFetchRecordZoneResult.clientChangeTokenData`.
    pub client_change_token_data: Option<Vec<u8>>,
    /// Mirrors `CKFetchRecordZoneResult.moreComing`.
    pub more_coming: bool,
    /// Mirrors `CKFetchRecordZoneResult.zoneError`.
    pub zone_error: Option<CloudKitError>,
}

/// Wraps `CKFetchRecordZoneChangesResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchRecordZoneChangesResult {
    /// Mirrors `CKFetchRecordZoneChangesResult.zones`.
    pub zones: Vec<CKFetchRecordZoneResult>,
    /// Mirrors `CKFetchRecordZoneChangesResult.operationError`.
    pub operation_error: Option<CloudKitError>,
}

impl CKFetchRecordZoneChangesResult {
    pub(crate) fn from_payload(payload: CKFetchRecordZoneChangesResultPayload) -> Self {
        Self {
            zones: payload
                .zones
                .into_iter()
                .map(|zone| CKFetchRecordZoneResult {
                    zone_id: CKRecordZoneID::from_payload(zone.zone_id),
                    changed_records: zone
                        .changed_records
                        .into_iter()
                        .map(CKRecord::from_payload)
                        .collect(),
                    deleted_records: zone
                        .deleted_records
                        .into_iter()
                        .map(CKDeletedRecord::from_payload)
                        .collect(),
                    updated_server_change_tokens: zone
                        .updated_server_change_tokens
                        .into_iter()
                        .map(CKServerChangeToken::from_payload)
                        .collect(),
                    server_change_token: zone
                        .server_change_token
                        .map(CKServerChangeToken::from_payload),
                    client_change_token_data: zone.client_change_token_data,
                    more_coming: zone.more_coming,
                    zone_error: zone.zone_error.map(CloudKitError::from_payload),
                })
                .collect(),
            operation_error: payload.operation_error.map(CloudKitError::from_payload),
        }
    }
}
