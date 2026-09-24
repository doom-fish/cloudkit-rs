import CloudKit
import Foundation

final class CKPermissionStatusCallbackBox: @unchecked Sendable {
    let callback: CKAccountStatusCallback
    let refcon: UnsafeMutableRawPointer?

    init(callback: @escaping CKAccountStatusCallback, refcon: UnsafeMutableRawPointer?) {
        self.callback = callback
        self.refcon = refcon
    }

    func complete(status: CKContainer.ApplicationPermissionStatus, error: NSError?) {
        let rawStatus = Int32(clamping: status.rawValue)
        if let error {
            ckErrorJSON(error).withCString { callback(refcon, rawStatus, $0) }
            return
        }
        callback(refcon, rawStatus, nil)
    }
}

private func ckDiscoverUserIdentity(
    container: CKContainer,
    lookupInfo: CKUserIdentityLookupInfoPayload,
    completion: @escaping (CKUserIdentity?, NSError?) -> Void
) throws {
    if let emailAddress = lookupInfo.emailAddress {
        container.discoverUserIdentity(withEmailAddress: emailAddress) { identity, error in
            completion(identity, error as NSError?)
        }
        return
    }
    if let phoneNumber = lookupInfo.phoneNumber {
        container.discoverUserIdentity(withPhoneNumber: phoneNumber) { identity, error in
            completion(identity, error as NSError?)
        }
        return
    }
    if let userRecordID = lookupInfo.userRecordID {
        container.discoverUserIdentity(withUserRecordID: try ckDecodeRecordID(userRecordID)) { identity, error in
            completion(identity, error as NSError?)
        }
        return
    }
    throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "User identity lookup info must include an email address, phone number, or user record ID")
}

private func ckEncodeModifyRecordsResult(
    request: CKModifyRecordsOperationPayload,
    originals: [CKRecord.ID: CKRecordPayload],
    saveResultsByID: [CKRecord.ID: Result<CKRecord, Error>] = [:],
    deleteResultsByID: [CKRecord.ID: Result<Void, Error>] = [:],
    operationError: NSError? = nil
) throws -> CKModifyRecordsResultPayload {
    var saves: [(CKRecord.ID, Result<CKRecord, Error>)] = []
    for requestedRecord in request.recordsToSave {
        let recordID = try ckDecodeRecordID(requestedRecord.recordID)
        if let saveResult = saveResultsByID[recordID] {
            saves.append((recordID, saveResult))
        }
    }
    var deletes: [(CKRecord.ID, Result<Void, Error>)] = []
    for requestedRecordID in request.recordIDsToDelete {
        let recordID = try ckDecodeRecordID(requestedRecordID)
        if let deleteResult = deleteResultsByID[recordID] {
            deletes.append((recordID, deleteResult))
        }
    }
    let outcome = CKModifyRecordsOutcome(
        saves: CKOperationOutcome(results: saves, operationError: operationError, timedOut: false),
        deletes: deletes
    )
    return ckEncodeModifyRecordsOutcome(outcome, originals: originals, label: "CKDatabase.modifyRecords", timeoutSeconds: -1)
}

private func ckEncodeDatabaseChangesResult(
    modifications: [CKDatabase.DatabaseChange.Modification],
    deletions: [CKDatabase.DatabaseChange.Deletion],
    changeToken: CKServerChangeToken?,
    moreComing: Bool,
    operationError: NSError?
) -> CKFetchDatabaseChangesResultPayload {
    var deletedZoneIDs: [CKRecordZoneIDPayload] = []
    var purgedZoneIDs: [CKRecordZoneIDPayload] = []
    var encryptedDataResetZoneIDs: [CKRecordZoneIDPayload] = []

    for deletion in deletions {
        let payload = ckEncodeZoneID(deletion.zoneID)
        if #available(macOS 14.0, *) {
            switch deletion.reason {
            case .deleted:
                deletedZoneIDs.append(payload)
            case .purged:
                purgedZoneIDs.append(payload)
            case .encryptedDataReset:
                encryptedDataResetZoneIDs.append(payload)
            @unknown default:
                deletedZoneIDs.append(payload)
            }
        } else if deletion.purged {
            purgedZoneIDs.append(payload)
        } else {
            deletedZoneIDs.append(payload)
        }
    }

    return CKFetchDatabaseChangesResultPayload(
        changedZoneIDs: modifications.map { ckEncodeZoneID($0.zoneID) },
        deletedZoneIDs: deletedZoneIDs,
        purgedZoneIDs: purgedZoneIDs,
        encryptedDataResetZoneIDs: encryptedDataResetZoneIDs,
        // The modern CKDatabase convenience API returns only the final token.
        updatedServerChangeTokens: [],
        serverChangeToken: changeToken.map(ckEncodeServerChangeToken),
        moreComing: moreComing,
        operationError: operationError.map(ckErrorPayload)
    )
}

@_cdecl("ck_container_request_application_permission_async")
public func ckContainerRequestApplicationPermissionAsync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ permissionRaw: Int32,
    _ callback: @escaping CKAccountStatusCallback,
    _ refcon: UnsafeMutableRawPointer?
) {
    let boxPtr = ckRetain(CKPermissionStatusCallbackBox(callback: callback, refcon: refcon))
    do {
        guard permissionRaw >= 0 else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "CloudKit application permission raw value must not be negative")
        }
        let container = try ckMakeContainer(containerIdentifier)
        let permission = CKContainer.ApplicationPermissions(rawValue: UInt(permissionRaw))
        container.requestApplicationPermission(permission) { status, error in
            let box: CKPermissionStatusCallbackBox = ckUnretained(boxPtr, as: CKPermissionStatusCallbackBox.self)
            defer { ckRelease(boxPtr) }
            box.complete(status: status, error: error as NSError?)
        }
    } catch let error as NSError {
        let box: CKPermissionStatusCallbackBox = ckUnretained(boxPtr, as: CKPermissionStatusCallbackBox.self)
        box.complete(status: .couldNotComplete, error: error)
        ckRelease(boxPtr)
    }
}

@_cdecl("ck_container_discover_user_identity_async")
public func ckContainerDiscoverUserIdentityAsync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ lookupInfoJSON: UnsafePointer<CChar>?,
    _ callback: @escaping CKJSONStringCallback,
    _ refcon: UnsafeMutableRawPointer?
) {
    let boxPtr = ckRetain(CKJSONStringCallbackBox(callback: callback, refcon: refcon))
    do {
        let container = try ckMakeContainer(containerIdentifier)
        let lookupInfo = try ckDecodeJSON(lookupInfoJSON, as: CKUserIdentityLookupInfoPayload.self)
        try ckDiscoverUserIdentity(container: container, lookupInfo: lookupInfo) { identity, error in
            let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
            defer { ckRelease(boxPtr) }
            if let error {
                box.fail(error: error)
            } else if let identity {
                do {
                    box.succeed(json: try ckEncodeJSON(ckEncodeUserIdentity(identity)))
                } catch let error as NSError {
                    box.fail(error: error)
                }
            } else {
                box.fail(error: ckBridgeNSError(code: CKR_FAILURE, message: "CloudKit returned a nil user identity"))
            }
        }
    } catch let error as NSError {
        let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
        box.fail(error: error)
        ckRelease(boxPtr)
    }
}

@_cdecl("ck_database_fetch_record_async")
public func ckDatabaseFetchRecordAsync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ recordIDJSON: UnsafePointer<CChar>?,
    _ callback: @escaping CKJSONStringCallback,
    _ refcon: UnsafeMutableRawPointer?
) {
    let boxPtr = ckRetain(CKJSONStringCallbackBox(callback: callback, refcon: refcon))
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let recordIDPayload = try ckDecodeJSON(recordIDJSON, as: CKRecordIDPayload.self)
        database.fetch(withRecordID: try ckDecodeRecordID(recordIDPayload)) { record, error in
            let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
            defer { ckRelease(boxPtr) }
            if let error = error as NSError? {
                box.fail(error: error)
            } else if let record {
                do {
                    box.succeed(json: try ckEncodeJSON(try ckEncodeRecord(record)))
                } catch let error as NSError {
                    box.fail(error: error)
                }
            } else {
                box.fail(error: ckBridgeNSError(code: CKR_FAILURE, message: "CloudKit returned a nil record"))
            }
        }
    } catch let error as NSError {
        let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
        box.fail(error: error)
        ckRelease(boxPtr)
    }
}

@_cdecl("ck_database_modify_records_async")
public func ckDatabaseModifyRecordsAsync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ operationJSON: UnsafePointer<CChar>?,
    _ callback: @escaping CKJSONStringCallback,
    _ refcon: UnsafeMutableRawPointer?
) {
    let boxPtr = ckRetain(CKJSONStringCallbackBox(callback: callback, refcon: refcon))
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let payload = try ckDecodeJSON(operationJSON, as: CKModifyRecordsOperationPayload.self)
        let savePolicy = try ckSavePolicy(payload.savePolicy)
        let (recordsToSave, originals) = try ckDecodeRecordsToSave(payload.recordsToSave, savePolicy: savePolicy)
        let recordIDsToDelete = try payload.recordIDsToDelete.map(ckDecodeRecordID)

        database.modifyRecords(
            saving: recordsToSave,
            deleting: recordIDsToDelete,
            savePolicy: savePolicy,
            atomically: payload.atomic
        ) { result in
            let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
            defer { ckRelease(boxPtr) }
            do {
                let encodedResult: CKModifyRecordsResultPayload
                switch result {
                case .success(let output):
                    encodedResult = try ckEncodeModifyRecordsResult(
                        request: payload,
                        originals: originals,
                        saveResultsByID: output.saveResults,
                        deleteResultsByID: output.deleteResults,
                        operationError: nil
                    )
                case .failure(let error):
                    encodedResult = try ckEncodeModifyRecordsResult(
                        request: payload,
                        originals: originals,
                        operationError: error as NSError
                    )
                }
                box.succeed(json: try ckEncodeJSON(encodedResult))
            } catch let error as NSError {
                box.fail(error: error)
            }
        }
    } catch let error as NSError {
        let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
        box.fail(error: error)
        ckRelease(boxPtr)
    }
}

@_cdecl("ck_database_delete_record_async")
public func ckDatabaseDeleteRecordAsync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ recordIDJSON: UnsafePointer<CChar>?,
    _ callback: @escaping CKJSONStringCallback,
    _ refcon: UnsafeMutableRawPointer?
) {
    let boxPtr = ckRetain(CKJSONStringCallbackBox(callback: callback, refcon: refcon))
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let recordIDPayload = try ckDecodeJSON(recordIDJSON, as: CKRecordIDPayload.self)
        database.delete(withRecordID: try ckDecodeRecordID(recordIDPayload)) { recordID, error in
            let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
            defer { ckRelease(boxPtr) }
            if let error = error as NSError? {
                box.fail(error: error)
            } else if let recordID {
                do {
                    box.succeed(json: try ckEncodeJSON(ckEncodeRecordID(recordID)))
                } catch let error as NSError {
                    box.fail(error: error)
                }
            } else {
                box.fail(error: ckBridgeNSError(code: CKR_FAILURE, message: "CloudKit returned a nil deleted record ID"))
            }
        }
    } catch let error as NSError {
        let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
        box.fail(error: error)
        ckRelease(boxPtr)
    }
}

@_cdecl("ck_database_fetch_query_results_async")
public func ckDatabaseFetchQueryResultsAsync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ queryJSON: UnsafePointer<CChar>?,
    _ cursorJSON: UnsafePointer<CChar>?,
    _ zoneIDJSON: UnsafePointer<CChar>?,
    _ desiredKeysJSON: UnsafePointer<CChar>?,
    _ resultsLimit: Int32,
    _ callback: @escaping CKJSONStringCallback,
    _ refcon: UnsafeMutableRawPointer?
) {
    let boxPtr = ckRetain(CKJSONStringCallbackBox(callback: callback, refcon: refcon))
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let cursor = try cursorJSON.map { try ckDecodeQueryCursor(try ckDecodeJSON($0, as: CKQueryCursorPayload.self)) }
        let query = try queryJSON.map { try ckDecodeQuery(try ckDecodeJSON($0, as: CKQueryPayload.self)) }
        let zoneID = try zoneIDJSON.map { try ckDecodeZoneID(try ckDecodeJSON($0, as: CKRecordZoneIDPayload.self)) }
        let desiredKeys = try desiredKeysJSON.map { try ckDecodeJSON($0, as: [String].self) }
        let limit = resultsLimit > 0 ? Int(resultsLimit) : CKQueryOperation.maximumResults

        let completion: (Result<(matchResults: [(CKRecord.ID, Result<CKRecord, Error>)], queryCursor: CKQueryOperation.Cursor?), Error>) -> Void = { result in
            let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
            defer { ckRelease(boxPtr) }
            switch result {
            case .success(let output):
                do {
                    let outcome = CKOperationOutcome(results: output.matchResults, operationError: nil, timedOut: false)
                    let (records, matches) = ckEncodeQueryMatches(outcome)
                    let payload = CKFetchedQueryResultsPayload(
                        records: records,
                        matches: matches,
                        cursor: try output.queryCursor.map(ckEncodeQueryCursor),
                        operationError: nil
                    )
                    box.succeed(json: try ckEncodeJSON(payload))
                } catch let error as NSError {
                    box.fail(error: error)
                }
            case .failure(let error):
                box.fail(error: error as NSError)
            }
        }

        if let cursor {
            database.fetch(withCursor: cursor, desiredKeys: desiredKeys, resultsLimit: limit, completionHandler: completion)
        } else if let query {
            database.fetch(withQuery: query, inZoneWith: zoneID, desiredKeys: desiredKeys, resultsLimit: limit, completionHandler: completion)
        } else {
            throw ckInvalidArgument("fetching query results needs a query or a cursor")
        }
    } catch let error as NSError {
        let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
        box.fail(error: error)
        ckRelease(boxPtr)
    }
}

@_cdecl("ck_database_fetch_changes_async")
public func ckDatabaseFetchChangesAsync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ operationJSON: UnsafePointer<CChar>?,
    _ callback: @escaping CKJSONStringCallback,
    _ refcon: UnsafeMutableRawPointer?
) {
    let boxPtr = ckRetain(CKJSONStringCallbackBox(callback: callback, refcon: refcon))
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let payload = try ckDecodeJSON(operationJSON, as: CKFetchDatabaseChangesOperationPayload.self)
        let previousToken = try payload.previousServerChangeToken.map(ckDecodeServerChangeToken)

        database.fetchDatabaseChanges(since: previousToken, resultsLimit: payload.resultsLimit) { result in
            let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
            defer { ckRelease(boxPtr) }
            do {
                let encodedResult: CKFetchDatabaseChangesResultPayload
                switch result {
                case .success(let output):
                    encodedResult = ckEncodeDatabaseChangesResult(
                        modifications: output.modifications,
                        deletions: output.deletions,
                        changeToken: output.changeToken,
                        moreComing: output.moreComing,
                        operationError: nil
                    )
                case .failure(let error):
                    encodedResult = ckEncodeDatabaseChangesResult(
                        modifications: [],
                        deletions: [],
                        changeToken: nil,
                        moreComing: false,
                        operationError: error as NSError
                    )
                }
                box.succeed(json: try ckEncodeJSON(encodedResult))
            } catch let error as NSError {
                box.fail(error: error)
            }
        }
    } catch let error as NSError {
        let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
        box.fail(error: error)
        ckRelease(boxPtr)
    }
}

@_cdecl("ck_database_fetch_all_record_zones_async")
public func ckDatabaseFetchAllRecordZonesAsync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ callback: @escaping CKJSONStringCallback,
    _ refcon: UnsafeMutableRawPointer?
) {
    let boxPtr = ckRetain(CKJSONStringCallbackBox(callback: callback, refcon: refcon))
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        database.fetchAllRecordZones { zones, error in
            let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
            defer { ckRelease(boxPtr) }
            if let error = error as NSError? {
                box.fail(error: error)
            } else if let zones {
                do {
                    box.succeed(json: try ckEncodeJSON(zones.map(ckEncodeZone)))
                } catch let error as NSError {
                    box.fail(error: error)
                }
            } else {
                box.fail(error: ckBridgeNSError(code: CKR_FAILURE, message: "CloudKit returned nil record zones"))
            }
        }
    } catch let error as NSError {
        let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
        box.fail(error: error)
        ckRelease(boxPtr)
    }
}
