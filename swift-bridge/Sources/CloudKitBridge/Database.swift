import CloudKit
import Foundation

private func ckRunModifyRecordZones(
    database: CKDatabase,
    zonesToSave: [CKRecordZone],
    zoneIDsToDelete: [CKRecordZone.ID],
    timeoutSeconds: Double
) -> (CKOperationOutcome<CKRecordZone.ID, CKRecordZone>, [(CKRecordZone.ID, Result<Void, Error>)]) {
    let operation = CKModifyRecordZonesOperation(recordZonesToSave: zonesToSave, recordZoneIDsToDelete: zoneIDsToDelete)
    operation.qualityOfService = .userInitiated
    let saves = CKItemResults<CKRecordZone.ID, CKRecordZone>()
    let deletes = CKItemResults<CKRecordZone.ID, Void>()
    operation.perRecordZoneSaveBlock = { zoneID, result in saves.append(zoneID, result) }
    operation.perRecordZoneDeleteBlock = { zoneID, result in deletes.append(zoneID, result) }
    operation.modifyRecordZonesResultBlock = { result in saves.finish(result) }
    let outcome = ckRunOperation(operation, on: database, results: saves, timeoutSeconds: timeoutSeconds)
    return (outcome, deletes.results)
}

private func ckRunFetchRecordZones(
    database: CKDatabase,
    zoneIDs: [CKRecordZone.ID]?,
    timeoutSeconds: Double
) -> CKOperationOutcome<CKRecordZone.ID, CKRecordZone> {
    let operation = zoneIDs.map { CKFetchRecordZonesOperation(recordZoneIDs: $0) }
        ?? CKFetchRecordZonesOperation.fetchAllRecordZonesOperation()
    operation.qualityOfService = .userInitiated
    let results = CKItemResults<CKRecordZone.ID, CKRecordZone>()
    operation.perRecordZoneResultBlock = { zoneID, result in results.append(zoneID, result) }
    operation.fetchRecordZonesResultBlock = { result in results.finish(result) }
    return ckRunOperation(operation, on: database, results: results, timeoutSeconds: timeoutSeconds)
}

private func ckRunModifySubscriptions(
    database: CKDatabase,
    subscriptionsToSave: [CKSubscription],
    subscriptionIDsToDelete: [CKSubscription.ID],
    timeoutSeconds: Double
) -> (CKOperationOutcome<CKSubscription.ID, CKSubscription>, [(CKSubscription.ID, Result<Void, Error>)]) {
    let operation = CKModifySubscriptionsOperation(
        subscriptionsToSave: subscriptionsToSave,
        subscriptionIDsToDelete: subscriptionIDsToDelete
    )
    operation.qualityOfService = .userInitiated
    let saves = CKItemResults<CKSubscription.ID, CKSubscription>()
    let deletes = CKItemResults<CKSubscription.ID, Void>()
    operation.perSubscriptionSaveBlock = { subscriptionID, result in saves.append(subscriptionID, result) }
    operation.perSubscriptionDeleteBlock = { subscriptionID, result in deletes.append(subscriptionID, result) }
    operation.modifySubscriptionsResultBlock = { result in saves.finish(result) }
    let outcome = ckRunOperation(operation, on: database, results: saves, timeoutSeconds: timeoutSeconds)
    return (outcome, deletes.results)
}

private func ckRunFetchSubscriptions(
    database: CKDatabase,
    subscriptionIDs: [CKSubscription.ID]?,
    timeoutSeconds: Double
) -> CKOperationOutcome<CKSubscription.ID, CKSubscription> {
    let operation = subscriptionIDs.map { CKFetchSubscriptionsOperation(subscriptionIDs: $0) }
        ?? CKFetchSubscriptionsOperation.fetchAllSubscriptionsOperation()
    operation.qualityOfService = .userInitiated
    let results = CKItemResults<CKSubscription.ID, CKSubscription>()
    operation.perSubscriptionResultBlock = { subscriptionID, result in results.append(subscriptionID, result) }
    operation.fetchSubscriptionsResultBlock = { result in results.finish(result) }
    return ckRunOperation(operation, on: database, results: results, timeoutSeconds: timeoutSeconds)
}

private func ckSingleDelete<Key>(
    _ outcome: CKOperationOutcome<Key, some Any>,
    _ deletes: [(Key, Result<Void, Error>)],
    label: String,
    timeoutSeconds: Double
) throws -> Key {
    if let (key, result) = deletes.first {
        switch result {
        case .success:
            return key
        case .failure(let error):
            if outcome.timedOut, let reported = outcome.reportedError(label: label, timeoutSeconds: timeoutSeconds, writes: true) {
                throw reported
            }
            throw error as NSError
        }
    }
    if let error = outcome.reportedError(label: label, timeoutSeconds: timeoutSeconds, writes: true) {
        throw error
    }
    throw ckBridgeNSError(code: CKR_FAILURE, message: "\(label) finished without a result")
}

private func ckAllItems<Key, Value>(
    _ outcome: CKOperationOutcome<Key, Value>,
    label: String,
    timeoutSeconds: Double
) throws -> [Value] {
    if let error = outcome.reportedError(label: label, timeoutSeconds: timeoutSeconds, writes: false) {
        throw error
    }
    return try outcome.results.map { _, result in
        switch result {
        case .success(let value):
            return value
        case .failure(let error):
            throw error as NSError
        }
    }
}

@_cdecl("ck_database_save_record_sync")
public func ckDatabaseSaveRecordSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ recordJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let payload = try ckDecodeJSON(recordJSON, as: CKRecordPayload.self)
        let outcome = ckRunModifyRecords(
            database: database,
            recordsToSave: [try ckDecodeRecord(payload, allFields: false)],
            recordIDsToDelete: [],
            savePolicy: .ifServerRecordUnchanged,
            atomic: false,
            qualityOfService: .userInitiated,
            timeoutSeconds: timeoutSeconds
        )
        let savedRecord = try outcome.saves.single(label: "CloudKit saveRecord", timeoutSeconds: timeoutSeconds, writes: true)
        do {
            outJSON?.pointee = ckCString(try ckEncodeJSON(try ckEncodeSavedRecord(savedRecord, original: payload)))
        } catch let error as NSError {
            throw ckBridgeNSError(
                code: CKR_FAILURE,
                message: "CloudKit saved the record, but the bridge could not encode the saved copy: \(error.localizedDescription)"
            )
        }
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_fetch_record_sync")
public func ckDatabaseFetchRecordSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ recordIDJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let recordIDPayload = try ckDecodeJSON(recordIDJSON, as: CKRecordIDPayload.self)
        let outcome = ckRunFetchRecords(
            database: database,
            recordIDs: [try ckDecodeRecordID(recordIDPayload)],
            desiredKeys: nil,
            qualityOfService: .userInitiated,
            timeoutSeconds: timeoutSeconds
        )
        let fetchedRecord = try outcome.single(label: "CloudKit fetchRecord", timeoutSeconds: timeoutSeconds, writes: false)
        outJSON?.pointee = ckCString(try ckEncodeJSON(try ckEncodeRecord(fetchedRecord)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_delete_record_sync")
public func ckDatabaseDeleteRecordSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ recordIDJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let recordIDPayload = try ckDecodeJSON(recordIDJSON, as: CKRecordIDPayload.self)
        let outcome = ckRunModifyRecords(
            database: database,
            recordsToSave: [],
            recordIDsToDelete: [try ckDecodeRecordID(recordIDPayload)],
            savePolicy: .ifServerRecordUnchanged,
            atomic: false,
            qualityOfService: .userInitiated,
            timeoutSeconds: timeoutSeconds
        )
        let deletedRecordID = try ckSingleDelete(outcome.saves, outcome.deletes, label: "CloudKit deleteRecord", timeoutSeconds: timeoutSeconds)
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeRecordID(deletedRecordID)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_perform_query_sync")
public func ckDatabasePerformQuerySync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ queryJSON: UnsafePointer<CChar>?,
    _ zoneIDJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let query = try ckDecodeQuery(try ckDecodeJSON(queryJSON, as: CKQueryPayload.self))
        let zoneID = try zoneIDJSON.map { try ckDecodeZoneID(try ckDecodeJSON($0, as: CKRecordZoneIDPayload.self)) }
        let (outcome, _) = try ckRunQuery(
            database: database,
            query: query,
            cursor: nil,
            zoneID: zoneID,
            desiredKeys: nil,
            resultsLimit: nil,
            qualityOfService: .userInitiated,
            timeoutSeconds: timeoutSeconds
        )
        if let error = outcome.reportedError(label: "CloudKit performQuery", timeoutSeconds: timeoutSeconds, writes: false) {
            throw error
        }
        let records = outcome.results.compactMap { try? $0.1.get() }
        outJSON?.pointee = ckCString(try ckEncodeJSON(try records.map(ckEncodeRecord)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_perform_query_async")
public func ckDatabasePerformQueryAsync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ queryJSON: UnsafePointer<CChar>?,
    _ zoneIDJSON: UnsafePointer<CChar>?,
    _ callback: @escaping CKJSONStringCallback,
    _ refcon: UnsafeMutableRawPointer?
) {
    let boxPtr = ckRetain(CKJSONStringCallbackBox(callback: callback, refcon: refcon))
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let query = try ckDecodeQuery(try ckDecodeJSON(queryJSON, as: CKQueryPayload.self))
        let zoneID = try zoneIDJSON.map { try ckDecodeZoneID(try ckDecodeJSON($0, as: CKRecordZoneIDPayload.self)) }
        database.fetch(withQuery: query, inZoneWith: zoneID, desiredKeys: nil, resultsLimit: CKQueryOperation.maximumResults) { result in
            let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
            defer { ckRelease(boxPtr) }
            switch result {
            case .success(let output):
                do {
                    let records = output.matchResults.compactMap { try? $0.1.get() }
                    box.succeed(json: try ckEncodeJSON(try records.map(ckEncodeRecord)))
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

@_cdecl("ck_database_fetch_query_results_sync")
public func ckDatabaseFetchQueryResultsSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ queryJSON: UnsafePointer<CChar>?,
    _ cursorJSON: UnsafePointer<CChar>?,
    _ zoneIDJSON: UnsafePointer<CChar>?,
    _ desiredKeysJSON: UnsafePointer<CChar>?,
    _ resultsLimit: Int32,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let cursor = try cursorJSON.map { try ckDecodeQueryCursor(try ckDecodeJSON($0, as: CKQueryCursorPayload.self)) }
        let query = try queryJSON.map { try ckDecodeQuery(try ckDecodeJSON($0, as: CKQueryPayload.self)) }
        let zoneID = try zoneIDJSON.map { try ckDecodeZoneID(try ckDecodeJSON($0, as: CKRecordZoneIDPayload.self)) }
        let desiredKeys = try desiredKeysJSON.map { try ckDecodeJSON($0, as: [String].self) }
        let (outcome, nextCursor) = try ckRunQuery(
            database: database,
            query: query,
            cursor: cursor,
            zoneID: zoneID,
            desiredKeys: desiredKeys,
            resultsLimit: resultsLimit > 0 ? Int(resultsLimit) : CKQueryOperation.maximumResults,
            qualityOfService: .userInitiated,
            timeoutSeconds: timeoutSeconds
        )
        if let error = outcome.reportedError(label: "CloudKit fetch query results", timeoutSeconds: timeoutSeconds, writes: false) {
            throw error
        }
        let (records, matches) = ckEncodeQueryMatches(outcome)
        let payload = CKFetchedQueryResultsPayload(
            records: records,
            matches: matches,
            cursor: try nextCursor.map(ckEncodeQueryCursor),
            operationError: nil
        )
        outJSON?.pointee = ckCString(try ckEncodeJSON(payload))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_fetch_all_record_zones_sync")
public func ckDatabaseFetchAllRecordZonesSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let outcome = ckRunFetchRecordZones(database: database, zoneIDs: nil, timeoutSeconds: timeoutSeconds)
        let zones = try ckAllItems(outcome, label: "CloudKit fetch all record zones", timeoutSeconds: timeoutSeconds)
        outJSON?.pointee = ckCString(try ckEncodeJSON(zones.map(ckEncodeZone)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_fetch_record_zone_sync")
public func ckDatabaseFetchRecordZoneSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ zoneIDJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let zoneID = try ckDecodeZoneID(try ckDecodeJSON(zoneIDJSON, as: CKRecordZoneIDPayload.self))
        let outcome = ckRunFetchRecordZones(database: database, zoneIDs: [zoneID], timeoutSeconds: timeoutSeconds)
        let zone = try outcome.single(label: "CloudKit fetch record zone", timeoutSeconds: timeoutSeconds, writes: false)
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeZone(zone)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_save_record_zone_sync")
public func ckDatabaseSaveRecordZoneSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ zoneJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let zone = try ckDecodeZone(try ckDecodeJSON(zoneJSON, as: CKRecordZonePayload.self))
        let (outcome, _) = ckRunModifyRecordZones(database: database, zonesToSave: [zone], zoneIDsToDelete: [], timeoutSeconds: timeoutSeconds)
        let savedZone = try outcome.single(label: "CloudKit save record zone", timeoutSeconds: timeoutSeconds, writes: true)
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeZone(savedZone)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_delete_record_zone_sync")
public func ckDatabaseDeleteRecordZoneSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ zoneIDJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let zoneID = try ckDecodeZoneID(try ckDecodeJSON(zoneIDJSON, as: CKRecordZoneIDPayload.self))
        let (outcome, deletes) = ckRunModifyRecordZones(database: database, zonesToSave: [], zoneIDsToDelete: [zoneID], timeoutSeconds: timeoutSeconds)
        let deletedZoneID = try ckSingleDelete(outcome, deletes, label: "CloudKit delete record zone", timeoutSeconds: timeoutSeconds)
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeZoneID(deletedZoneID)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_fetch_subscription_sync")
public func ckDatabaseFetchSubscriptionSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ subscriptionID: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard let subscriptionID else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Missing subscription ID")
        }
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let outcome = ckRunFetchSubscriptions(database: database, subscriptionIDs: [String(cString: subscriptionID)], timeoutSeconds: timeoutSeconds)
        let subscription = try outcome.single(label: "CloudKit fetch subscription", timeoutSeconds: timeoutSeconds, writes: false)
        outJSON?.pointee = ckCString(try ckEncodeJSON(try ckEncodeSubscription(subscription)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_fetch_all_subscriptions_sync")
public func ckDatabaseFetchAllSubscriptionsSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let outcome = ckRunFetchSubscriptions(database: database, subscriptionIDs: nil, timeoutSeconds: timeoutSeconds)
        let subscriptions = try ckAllItems(outcome, label: "CloudKit fetch all subscriptions", timeoutSeconds: timeoutSeconds)
        outJSON?.pointee = ckCString(try ckEncodeJSON(try subscriptions.map(ckEncodeSubscription)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_save_subscription_sync")
public func ckDatabaseSaveSubscriptionSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ subscriptionJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let subscription = try ckDecodeSubscription(try ckDecodeJSON(subscriptionJSON, as: CKSubscriptionPayload.self))
        let (outcome, _) = ckRunModifySubscriptions(
            database: database,
            subscriptionsToSave: [subscription],
            subscriptionIDsToDelete: [],
            timeoutSeconds: timeoutSeconds
        )
        let savedSubscription = try outcome.single(label: "CloudKit save subscription", timeoutSeconds: timeoutSeconds, writes: true)
        outJSON?.pointee = ckCString(try ckEncodeJSON(try ckEncodeSubscription(savedSubscription)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_delete_subscription_sync")
public func ckDatabaseDeleteSubscriptionSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ subscriptionID: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard let subscriptionID else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Missing subscription ID")
        }
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let (outcome, deletes) = ckRunModifySubscriptions(
            database: database,
            subscriptionsToSave: [],
            subscriptionIDsToDelete: [String(cString: subscriptionID)],
            timeoutSeconds: timeoutSeconds
        )
        let deletedSubscriptionID = try ckSingleDelete(outcome, deletes, label: "CloudKit delete subscription", timeoutSeconds: timeoutSeconds)
        outJSON?.pointee = ckCString(try ckEncodeJSON(deletedSubscriptionID))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}
