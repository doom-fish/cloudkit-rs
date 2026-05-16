import CloudKit
import Foundation

struct CKFetchedQueryResultsPayload: Codable {
    var records: [CKRecordPayload]
    var matches: [CKQueryMatchResultPayload]
    var cursor: CKQueryCursorPayload?
    var operationError: CKErrorPayload?
}

struct CKFetchRecordsOperationPayload: Codable {
    var recordIDs: [CKRecordIDPayload]
    var desiredKeys: [String]?
}

struct CKRecordResultPayload: Codable {
    var recordID: CKRecordIDPayload
    var record: CKRecordPayload?
    var error: CKErrorPayload?
}

struct CKFetchRecordsResultPayload: Codable {
    var records: [CKRecordPayload]
    var results: [CKRecordResultPayload]
    var operationError: CKErrorPayload?
}

struct CKFetchDatabaseChangesOperationPayload: Codable {
    var previousServerChangeToken: CKServerChangeTokenPayload?
    var resultsLimit: Int?
    var fetchAllChanges: Bool
}

struct CKFetchDatabaseChangesResultPayload: Codable {
    var changedZoneIDs: [CKRecordZoneIDPayload]
    var deletedZoneIDs: [CKRecordZoneIDPayload]
    var purgedZoneIDs: [CKRecordZoneIDPayload]
    var encryptedDataResetZoneIDs: [CKRecordZoneIDPayload]
    var updatedServerChangeTokens: [CKServerChangeTokenPayload]
    var serverChangeToken: CKServerChangeTokenPayload?
    var moreComing: Bool
    var operationError: CKErrorPayload?
}

struct CKFetchRecordZoneChangesConfigurationPayload: Codable {
    var previousServerChangeToken: CKServerChangeTokenPayload?
    var resultsLimit: Int?
    var desiredKeys: [String]?
}

struct CKFetchRecordZoneChangesConfigurationEntryPayload: Codable {
    var zoneID: CKRecordZoneIDPayload
    var configuration: CKFetchRecordZoneChangesConfigurationPayload
}

struct CKDeletedRecordPayload: Codable {
    var recordID: CKRecordIDPayload
    var recordType: String
}

struct CKFetchRecordZoneResultPayload: Codable {
    var zoneID: CKRecordZoneIDPayload
    var changedRecords: [CKRecordPayload]
    var deletedRecords: [CKDeletedRecordPayload]
    var updatedServerChangeTokens: [CKServerChangeTokenPayload]
    var serverChangeToken: CKServerChangeTokenPayload?
    var clientChangeTokenData: [UInt8]?
    var moreComing: Bool
    var zoneError: CKErrorPayload?
}

struct CKFetchRecordZoneChangesOperationPayload: Codable {
    var zones: [CKFetchRecordZoneChangesConfigurationEntryPayload]
    var fetchAllChanges: Bool
}

struct CKFetchRecordZoneChangesResultPayload: Codable {
    var zones: [CKFetchRecordZoneResultPayload]
    var operationError: CKErrorPayload?
}
