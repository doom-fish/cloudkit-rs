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
        if let error {
            let payload = (try? ckEncodeJSON(ckErrorPayload(from: error))) ?? "{\"domain\":\"\(error.domain)\",\"code\":\(error.code),\"message\":\"\(error.localizedDescription)\",\"retryAfterSeconds\":null}"
            payload.withCString { callback(refcon, Int32(status.rawValue), $0) }
            return
        }
        callback(refcon, Int32(status.rawValue), nil)
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
        container.discoverUserIdentity(withUserRecordID: ckDecodeRecordID(userRecordID)) { identity, error in
            completion(identity, error as NSError?)
        }
        return
    }
    throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "User identity lookup info must include an email address, phone number, or user record ID")
}

private func ckEncodeModifyRecordsResult(
    request: CKModifyRecordsOperationPayload,
    saveResultsByID: [CKRecord.ID: Result<CKRecord, Error>] = [:],
    deleteResultsByID: [CKRecord.ID: Result<Void, Error>] = [:],
    operationError: NSError? = nil
) -> CKModifyRecordsResultPayload {
    var savedRecords: [CKRecordPayload] = []
    var deletedRecordIDs: [CKRecordIDPayload] = []
    var saveResults: [CKRecordSaveResultPayload] = []
    var deleteResults: [CKRecordDeleteResultPayload] = []

    for requestedRecord in request.recordsToSave {
        let recordID = ckDecodeRecordID(requestedRecord.recordID)
        guard let saveResult = saveResultsByID[recordID] else { continue }
        switch saveResult {
        case .success(let record):
            let payload = try? ckEncodeRecord(record)
            if let payload {
                savedRecords.append(payload)
            }
            saveResults.append(
                CKRecordSaveResultPayload(
                    recordID: ckEncodeRecordID(recordID),
                    record: payload,
                    error: nil
                )
            )
        case .failure(let error):
            saveResults.append(
                CKRecordSaveResultPayload(
                    recordID: ckEncodeRecordID(recordID),
                    record: nil,
                    error: ckErrorPayload(from: error as NSError)
                )
            )
        }
    }

    for requestedRecordID in request.recordIDsToDelete {
        let recordID = ckDecodeRecordID(requestedRecordID)
        guard let deleteResult = deleteResultsByID[recordID] else { continue }
        switch deleteResult {
        case .success:
            deletedRecordIDs.append(ckEncodeRecordID(recordID))
            deleteResults.append(
                CKRecordDeleteResultPayload(
                    recordID: ckEncodeRecordID(recordID),
                    error: nil
                )
            )
        case .failure(let error):
            deleteResults.append(
                CKRecordDeleteResultPayload(
                    recordID: ckEncodeRecordID(recordID),
                    error: ckErrorPayload(from: error as NSError)
                )
            )
        }
    }

    return CKModifyRecordsResultPayload(
        savedRecords: savedRecords,
        deletedRecordIDs: deletedRecordIDs,
        saveResults: saveResults,
        deleteResults: deleteResults,
        operationError: operationError.map(ckErrorPayload)
    )
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
        database.fetch(withRecordID: ckDecodeRecordID(recordIDPayload)) { record, error in
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
        let recordsToSave = try payload.recordsToSave.map(ckDecodeRecord)
        let recordIDsToDelete = payload.recordIDsToDelete.map(ckDecodeRecordID)
        let savePolicy = CKModifyRecordsOperation.RecordSavePolicy(rawValue: Int(payload.savePolicy)) ?? .ifServerRecordUnchanged

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
                    encodedResult = ckEncodeModifyRecordsResult(
                        request: payload,
                        saveResultsByID: output.saveResults,
                        deleteResultsByID: output.deleteResults,
                        operationError: nil
                    )
                case .failure(let error):
                    encodedResult = ckEncodeModifyRecordsResult(
                        request: payload,
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
        database.delete(withRecordID: ckDecodeRecordID(recordIDPayload)) { recordID, error in
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
    _ zoneIDJSON: UnsafePointer<CChar>?,
    _ desiredKeysJSON: UnsafePointer<CChar>?,
    _ resultsLimit: Int32,
    _ callback: @escaping CKJSONStringCallback,
    _ refcon: UnsafeMutableRawPointer?
) {
    let boxPtr = ckRetain(CKJSONStringCallbackBox(callback: callback, refcon: refcon))
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let query = ckDecodeQuery(try ckDecodeJSON(queryJSON, as: CKQueryPayload.self))
        let zoneID = try zoneIDJSON.map { try ckDecodeJSON($0, as: CKRecordZoneIDPayload.self) }.map(ckDecodeZoneID)
        let desiredKeys = try desiredKeysJSON.map { try ckDecodeJSON($0, as: [String].self) }
        let limit = resultsLimit > 0 ? Int(resultsLimit) : CKQueryOperation.maximumResults

        database.fetch(withQuery: query, inZoneWith: zoneID, desiredKeys: desiredKeys, resultsLimit: limit) { result in
            let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
            defer { ckRelease(boxPtr) }
            switch result {
            case .success(let output):
                do {
                    let records = try output.matchResults.compactMap { try? $0.1.get() }.map(ckEncodeRecord)
                    let matches = try output.matchResults.map { recordID, recordResult in
                        switch recordResult {
                        case .success(let record):
                            return CKQueryMatchResultPayload(
                                recordID: ckEncodeRecordID(recordID),
                                record: try ckEncodeRecord(record),
                                error: nil
                            )
                        case .failure(let error):
                            return CKQueryMatchResultPayload(
                                recordID: ckEncodeRecordID(recordID),
                                record: nil,
                                error: ckErrorPayload(from: error as NSError)
                            )
                        }
                    }
                    let payload = CKFetchedQueryResultsPayload(
                        records: records,
                        matches: matches,
                        cursor: output.queryCursor.map(ckEncodeQueryCursor),
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
