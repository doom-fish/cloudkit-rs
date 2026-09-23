import CloudKit
import Foundation

final class CKItemResults<Key, Value>: @unchecked Sendable {
    private let lock = NSLock()
    private var _results: [(Key, Result<Value, Error>)] = []
    private var _operationError: NSError?
    let finished = DispatchSemaphore(value: 0)

    func append(_ key: Key, _ result: Result<Value, Error>) {
        lock.lock()
        _results.append((key, result))
        lock.unlock()
    }

    func finish(_ result: Result<Void, Error>) {
        lock.lock()
        if case .failure(let error) = result {
            _operationError = error as NSError
        }
        lock.unlock()
        finished.signal()
    }

    var results: [(Key, Result<Value, Error>)] {
        lock.lock()
        defer { lock.unlock() }
        return _results
    }

    var operationError: NSError? {
        lock.lock()
        defer { lock.unlock() }
        return _operationError
    }
}

struct CKOperationOutcome<Key, Value> {
    var results: [(Key, Result<Value, Error>)]
    var operationError: NSError?
    var timedOut: Bool

    func reportedError(label: String, timeoutSeconds: Double, writes: Bool) -> NSError? {
        guard timedOut else { return operationError }
        if writes {
            return ckCancelledAfterTimeoutNSError(label, timeoutSeconds: timeoutSeconds, underlying: operationError)
        }
        return ckTimeoutNSError(label, timeoutSeconds: timeoutSeconds)
    }

    func single(label: String, timeoutSeconds: Double, writes: Bool) throws -> Value {
        if let (_, result) = results.first {
            switch result {
            case .success(let value):
                return value
            case .failure(let error):
                if timedOut {
                    throw reportedError(label: label, timeoutSeconds: timeoutSeconds, writes: writes) ?? error as NSError
                }
                throw error as NSError
            }
        }
        if let error = reportedError(label: label, timeoutSeconds: timeoutSeconds, writes: writes) {
            throw error
        }
        throw ckBridgeNSError(code: CKR_FAILURE, message: "\(label) finished without a result")
    }
}

func ckRunOperation<Key, Value>(
    _ operation: CKDatabaseOperation,
    on database: CKDatabase,
    results: CKItemResults<Key, Value>,
    timeoutSeconds: Double
) -> CKOperationOutcome<Key, Value> {
    database.add(operation)
    let timedOut = ckWaitCancellingOnTimeout(operation, results.finished, timeoutSeconds: timeoutSeconds)
    return CKOperationOutcome(results: results.results, operationError: results.operationError, timedOut: timedOut)
}

func ckSavePolicy(_ rawValue: Int32) throws -> CKModifyRecordsOperation.RecordSavePolicy {
    switch Int(rawValue) {
    case CKModifyRecordsOperation.RecordSavePolicy.ifServerRecordUnchanged.rawValue:
        return .ifServerRecordUnchanged
    case CKModifyRecordsOperation.RecordSavePolicy.changedKeys.rawValue:
        return .changedKeys
    case CKModifyRecordsOperation.RecordSavePolicy.allKeys.rawValue:
        return .allKeys
    default:
        throw ckInvalidArgument("Unsupported CKModifyRecordsOperation.RecordSavePolicy raw value \(rawValue)")
    }
}

struct CKModifyRecordsOutcome {
    var saves: CKOperationOutcome<CKRecord.ID, CKRecord>
    var deletes: [(CKRecord.ID, Result<Void, Error>)]
}

func ckRunModifyRecords(
    database: CKDatabase,
    recordsToSave: [CKRecord],
    recordIDsToDelete: [CKRecord.ID],
    savePolicy: CKModifyRecordsOperation.RecordSavePolicy,
    atomic: Bool,
    qualityOfService: QualityOfService?,
    timeoutSeconds: Double
) -> CKModifyRecordsOutcome {
    let operation = CKModifyRecordsOperation(recordsToSave: recordsToSave, recordIDsToDelete: recordIDsToDelete)
    operation.savePolicy = savePolicy
    operation.isAtomic = atomic
    if let qualityOfService {
        operation.qualityOfService = qualityOfService
    }
    let saves = CKItemResults<CKRecord.ID, CKRecord>()
    let deletes = CKItemResults<CKRecord.ID, Void>()
    operation.perRecordSaveBlock = { recordID, result in saves.append(recordID, result) }
    operation.perRecordDeleteBlock = { recordID, result in deletes.append(recordID, result) }
    operation.modifyRecordsResultBlock = { result in saves.finish(result) }
    let outcome = ckRunOperation(operation, on: database, results: saves, timeoutSeconds: timeoutSeconds)
    return CKModifyRecordsOutcome(saves: outcome, deletes: deletes.results)
}

func ckEncodeModifyRecordsOutcome(
    _ outcome: CKModifyRecordsOutcome,
    label: String,
    timeoutSeconds: Double
) -> CKModifyRecordsResultPayload {
    var savedRecords: [CKRecordPayload] = []
    var saveResults: [CKRecordSaveResultPayload] = []
    for (recordID, result) in outcome.saves.results {
        switch result {
        case .success(let record):
            let (payload, encodeError) = ckEncodeRecordResult(record)
            if let payload {
                savedRecords.append(payload)
            }
            saveResults.append(
                CKRecordSaveResultPayload(
                    recordID: ckEncodeRecordID(recordID),
                    record: payload,
                    error: encodeError.map {
                        CKErrorPayload(
                            domain: $0.domain,
                            code: $0.code,
                            message: "CloudKit saved the record, but the bridge could not encode the saved copy: \($0.message)",
                            retryAfterSeconds: nil
                        )
                    }
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

    var deletedRecordIDs: [CKRecordIDPayload] = []
    var deleteResults: [CKRecordDeleteResultPayload] = []
    for (recordID, result) in outcome.deletes {
        switch result {
        case .success:
            deletedRecordIDs.append(ckEncodeRecordID(recordID))
            deleteResults.append(CKRecordDeleteResultPayload(recordID: ckEncodeRecordID(recordID), error: nil))
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
        operationError: outcome.saves.reportedError(label: label, timeoutSeconds: timeoutSeconds, writes: true).map(ckErrorPayload)
    )
}

func ckRunFetchRecords(
    database: CKDatabase,
    recordIDs: [CKRecord.ID],
    desiredKeys: [String]?,
    qualityOfService: QualityOfService?,
    timeoutSeconds: Double
) -> CKOperationOutcome<CKRecord.ID, CKRecord> {
    let operation = CKFetchRecordsOperation(recordIDs: recordIDs)
    operation.desiredKeys = desiredKeys
    if let qualityOfService {
        operation.qualityOfService = qualityOfService
    }
    let results = CKItemResults<CKRecord.ID, CKRecord>()
    operation.perRecordResultBlock = { recordID, result in results.append(recordID, result) }
    operation.fetchRecordsResultBlock = { result in results.finish(result) }
    return ckRunOperation(operation, on: database, results: results, timeoutSeconds: timeoutSeconds)
}

final class CKQueryResults: @unchecked Sendable {
    let items = CKItemResults<CKRecord.ID, CKRecord>()
    private let lock = NSLock()
    private var _cursor: CKQueryOperation.Cursor?

    var cursor: CKQueryOperation.Cursor? {
        get { lock.lock(); defer { lock.unlock() }; return _cursor }
        set { lock.lock(); defer { lock.unlock() }; _cursor = newValue }
    }
}

func ckRunQuery(
    database: CKDatabase,
    query: CKQuery?,
    cursor: CKQueryOperation.Cursor?,
    zoneID: CKRecordZone.ID?,
    desiredKeys: [String]?,
    resultsLimit: Int?,
    qualityOfService: QualityOfService?,
    timeoutSeconds: Double
) throws -> (CKOperationOutcome<CKRecord.ID, CKRecord>, CKQueryOperation.Cursor?) {
    let operation: CKQueryOperation
    if let cursor {
        operation = CKQueryOperation(cursor: cursor)
    } else if let query {
        operation = CKQueryOperation(query: query)
        operation.zoneID = zoneID
    } else {
        throw ckInvalidArgument("A CKQueryOperation needs a query or a cursor")
    }
    operation.desiredKeys = desiredKeys
    if let resultsLimit {
        guard resultsLimit >= 0 else {
            throw ckInvalidArgument("CKQueryOperation.resultsLimit must not be negative")
        }
        operation.resultsLimit = resultsLimit
    }
    if let qualityOfService {
        operation.qualityOfService = qualityOfService
    }
    let results = CKQueryResults()
    operation.recordMatchedBlock = { recordID, result in results.items.append(recordID, result) }
    operation.queryResultBlock = { result in
        switch result {
        case .success(let cursor):
            results.cursor = cursor
            results.items.finish(.success(()))
        case .failure(let error):
            results.items.finish(.failure(error))
        }
    }
    let outcome = ckRunOperation(operation, on: database, results: results.items, timeoutSeconds: timeoutSeconds)
    return (outcome, results.cursor)
}

func ckEncodeQueryMatches(
    _ outcome: CKOperationOutcome<CKRecord.ID, CKRecord>
) -> ([CKRecordPayload], [CKQueryMatchResultPayload]) {
    var records: [CKRecordPayload] = []
    var matches: [CKQueryMatchResultPayload] = []
    for (recordID, result) in outcome.results {
        switch result {
        case .success(let record):
            let (payload, encodeError) = ckEncodeRecordResult(record)
            if let payload {
                records.append(payload)
            }
            matches.append(CKQueryMatchResultPayload(recordID: ckEncodeRecordID(recordID), record: payload, error: encodeError))
        case .failure(let error):
            matches.append(
                CKQueryMatchResultPayload(
                    recordID: ckEncodeRecordID(recordID),
                    record: nil,
                    error: ckErrorPayload(from: error as NSError)
                )
            )
        }
    }
    return (records, matches)
}

@_cdecl("ck_database_execute_modify_records_sync")
public func ckDatabaseExecuteModifyRecordsSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ operationJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let payload = try ckDecodeJSON(operationJSON, as: CKModifyRecordsOperationPayload.self)
        let outcome = ckRunModifyRecords(
            database: database,
            recordsToSave: try payload.recordsToSave.map(ckDecodeRecord),
            recordIDsToDelete: try payload.recordIDsToDelete.map(ckDecodeRecordID),
            savePolicy: try ckSavePolicy(payload.savePolicy),
            atomic: payload.atomic,
            qualityOfService: nil,
            timeoutSeconds: timeoutSeconds
        )
        let result = ckEncodeModifyRecordsOutcome(outcome, label: "CKModifyRecordsOperation", timeoutSeconds: timeoutSeconds)
        outJSON?.pointee = ckCString(try ckEncodeJSON(result))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_execute_query_operation_sync")
public func ckDatabaseExecuteQueryOperationSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ operationJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let payload = try ckDecodeJSON(operationJSON, as: CKQueryOperationPayload.self)
        let cursor = try payload.cursor.map(ckDecodeQueryCursor)
        let (outcome, nextCursor) = try ckRunQuery(
            database: database,
            query: cursor == nil ? try ckDecodeQuery(payload.query) : nil,
            cursor: cursor,
            zoneID: try payload.zoneID.map(ckDecodeZoneID),
            desiredKeys: payload.desiredKeys,
            resultsLimit: payload.resultsLimit,
            qualityOfService: nil,
            timeoutSeconds: timeoutSeconds
        )
        let (records, matches) = ckEncodeQueryMatches(outcome)
        let result = CKQueryOperationResultPayload(
            records: records,
            matches: matches,
            cursorReturned: nextCursor != nil,
            cursor: try nextCursor.map(ckEncodeQueryCursor),
            operationError: outcome.reportedError(label: "CKQueryOperation", timeoutSeconds: timeoutSeconds, writes: false).map(ckErrorPayload)
        )
        outJSON?.pointee = ckCString(try ckEncodeJSON(result))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_execute_fetch_records_sync")
public func ckDatabaseExecuteFetchRecordsSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ operationJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let payload = try ckDecodeJSON(operationJSON, as: CKFetchRecordsOperationPayload.self)
        let outcome = ckRunFetchRecords(
            database: database,
            recordIDs: try payload.recordIDs.map(ckDecodeRecordID),
            desiredKeys: payload.desiredKeys,
            qualityOfService: nil,
            timeoutSeconds: timeoutSeconds
        )
        let (records, matches) = ckEncodeQueryMatches(outcome)
        let result = CKFetchRecordsResultPayload(
            records: records,
            results: matches.map { CKRecordResultPayload(recordID: $0.recordID, record: $0.record, error: $0.error) },
            operationError: outcome.reportedError(label: "CKFetchRecordsOperation", timeoutSeconds: timeoutSeconds, writes: false).map(ckErrorPayload)
        )
        outJSON?.pointee = ckCString(try ckEncodeJSON(result))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

final class CKDatabaseChangesCollector: @unchecked Sendable {
    private let lock = NSLock()
    var changedZoneIDs: [CKRecordZoneIDPayload] = []
    var deletedZoneIDs: [CKRecordZoneIDPayload] = []
    var purgedZoneIDs: [CKRecordZoneIDPayload] = []
    var encryptedDataResetZoneIDs: [CKRecordZoneIDPayload] = []
    var updatedServerChangeTokens: [CKServerChangeTokenPayload] = []
    var result: CKFetchDatabaseChangesResultPayload?
    let finished = DispatchSemaphore(value: 0)

    func withLock<T>(_ body: (CKDatabaseChangesCollector) -> T) -> T {
        lock.lock()
        defer { lock.unlock() }
        return body(self)
    }
}

@_cdecl("ck_database_execute_fetch_database_changes_sync")
public func ckDatabaseExecuteFetchDatabaseChangesSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ operationJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let payload = try ckDecodeJSON(operationJSON, as: CKFetchDatabaseChangesOperationPayload.self)
        let operation = CKFetchDatabaseChangesOperation(previousServerChangeToken: try payload.previousServerChangeToken.map(ckDecodeServerChangeToken))
        if let resultsLimit = payload.resultsLimit {
            operation.resultsLimit = resultsLimit
        }
        operation.fetchAllChanges = payload.fetchAllChanges

        let collector = CKDatabaseChangesCollector()
        operation.recordZoneWithIDChangedBlock = { zoneID in
            collector.withLock { $0.changedZoneIDs.append(ckEncodeZoneID(zoneID)) }
        }
        operation.recordZoneWithIDWasDeletedBlock = { zoneID in
            collector.withLock { $0.deletedZoneIDs.append(ckEncodeZoneID(zoneID)) }
        }
        operation.recordZoneWithIDWasPurgedBlock = { zoneID in
            collector.withLock { $0.purgedZoneIDs.append(ckEncodeZoneID(zoneID)) }
        }
        operation.recordZoneWithIDWasDeletedDueToUserEncryptedDataResetBlock = { zoneID in
            collector.withLock { $0.encryptedDataResetZoneIDs.append(ckEncodeZoneID(zoneID)) }
        }
        operation.changeTokenUpdatedBlock = { serverChangeToken in
            collector.withLock { $0.updatedServerChangeTokens.append(ckEncodeServerChangeToken(serverChangeToken)) }
        }
        operation.fetchDatabaseChangesResultBlock = { operationResult in
            collector.withLock { collector in
                let serverChangeToken: CKServerChangeToken?
                let moreComing: Bool
                let operationError: CKErrorPayload?
                switch operationResult {
                case .success(let output):
                    serverChangeToken = output.serverChangeToken
                    moreComing = output.moreComing
                    operationError = nil
                case .failure(let error):
                    serverChangeToken = nil
                    moreComing = false
                    operationError = ckErrorPayload(from: error as NSError)
                }
                collector.result = CKFetchDatabaseChangesResultPayload(
                    changedZoneIDs: collector.changedZoneIDs,
                    deletedZoneIDs: collector.deletedZoneIDs,
                    purgedZoneIDs: collector.purgedZoneIDs,
                    encryptedDataResetZoneIDs: collector.encryptedDataResetZoneIDs,
                    updatedServerChangeTokens: collector.updatedServerChangeTokens,
                    serverChangeToken: serverChangeToken.map(ckEncodeServerChangeToken),
                    moreComing: moreComing,
                    operationError: operationError
                )
            }
            collector.finished.signal()
        }

        database.add(operation)
        let timedOut = ckWaitCancellingOnTimeout(operation, collector.finished, timeoutSeconds: timeoutSeconds)
        guard var result = collector.withLock({ $0.result }) else {
            throw ckBridgeNSError(code: CKR_FAILURE, message: "CKFetchDatabaseChangesOperation completed without a result payload")
        }
        if timedOut {
            result.operationError = ckErrorPayload(from: ckTimeoutNSError("CKFetchDatabaseChangesOperation", timeoutSeconds: timeoutSeconds))
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(result))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

final class CKZoneChangesCollector: @unchecked Sendable {
    private let lock = NSLock()
    private var zoneResults: [String: CKFetchRecordZoneResultPayload] = [:]
    private var encodeError: NSError?
    private var operationError: NSError?
    let finished = DispatchSemaphore(value: 0)

    func update(_ zoneID: CKRecordZone.ID, _ body: (inout CKFetchRecordZoneResultPayload) -> Void) {
        lock.lock()
        defer { lock.unlock() }
        let key = "\(zoneID.zoneName)|\(zoneID.ownerName)"
        var result = zoneResults[key] ?? CKFetchRecordZoneResultPayload(
            zoneID: ckEncodeZoneID(zoneID),
            changedRecords: [],
            deletedRecords: [],
            updatedServerChangeTokens: [],
            serverChangeToken: nil,
            clientChangeTokenData: nil,
            moreComing: false,
            zoneError: nil
        )
        body(&result)
        zoneResults[key] = result
    }

    func recordEncodeFailure(_ error: NSError) {
        lock.lock()
        if encodeError == nil {
            encodeError = error
        }
        lock.unlock()
    }

    func finish(_ error: Error?) {
        lock.lock()
        operationError = error as NSError?
        lock.unlock()
        finished.signal()
    }

    func snapshot() -> ([CKFetchRecordZoneResultPayload], NSError?, NSError?) {
        lock.lock()
        defer { lock.unlock() }
        let zones = zoneResults.values.sorted {
            ($0.zoneID.zoneName, $0.zoneID.ownerName) < ($1.zoneID.zoneName, $1.zoneID.ownerName)
        }
        return (zones, encodeError, operationError)
    }
}

@_cdecl("ck_database_execute_fetch_record_zone_changes_sync")
public func ckDatabaseExecuteFetchRecordZoneChangesSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ operationJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let payload = try ckDecodeJSON(operationJSON, as: CKFetchRecordZoneChangesOperationPayload.self)

        var zoneIDs: [CKRecordZone.ID] = []
        var configurations: [CKRecordZone.ID: CKFetchRecordZoneChangesOperation.ZoneConfiguration] = [:]
        for entry in payload.zones {
            let zoneID = try ckDecodeZoneID(entry.zoneID)
            let configuration = CKFetchRecordZoneChangesOperation.ZoneConfiguration()
            configuration.previousServerChangeToken = try entry.configuration.previousServerChangeToken.map(ckDecodeServerChangeToken)
            if let resultsLimit = entry.configuration.resultsLimit {
                configuration.resultsLimit = resultsLimit
            }
            configuration.desiredKeys = entry.configuration.desiredKeys
            if configurations.updateValue(configuration, forKey: zoneID) == nil {
                zoneIDs.append(zoneID)
            }
        }
        let operation = CKFetchRecordZoneChangesOperation(recordZoneIDs: zoneIDs, configurationsByRecordZoneID: configurations)
        operation.fetchAllChanges = payload.fetchAllChanges

        let collector = CKZoneChangesCollector()
        operation.recordChangedBlock = { record in
            do {
                let encoded = try ckEncodeRecord(record)
                collector.update(record.recordID.zoneID) { $0.changedRecords.append(encoded) }
            } catch let error as NSError {
                collector.recordEncodeFailure(error)
            }
        }
        operation.recordWithIDWasDeletedBlock = { recordID, recordType in
            collector.update(recordID.zoneID) { result in
                result.deletedRecords.append(
                    CKDeletedRecordPayload(recordID: ckEncodeRecordID(recordID), recordType: recordType)
                )
            }
        }
        operation.recordZoneChangeTokensUpdatedBlock = { zoneID, serverChangeToken, clientChangeTokenData in
            collector.update(zoneID) { result in
                if let serverChangeToken {
                    result.updatedServerChangeTokens.append(ckEncodeServerChangeToken(serverChangeToken))
                }
                result.clientChangeTokenData = clientChangeTokenData.map { [UInt8]($0) }
            }
        }
        operation.recordZoneFetchCompletionBlock = { zoneID, serverChangeToken, clientChangeTokenData, moreComing, error in
            collector.update(zoneID) { result in
                result.serverChangeToken = serverChangeToken.map(ckEncodeServerChangeToken)
                result.clientChangeTokenData = clientChangeTokenData.map { [UInt8]($0) }
                result.moreComing = moreComing
                result.zoneError = (error as NSError?).map(ckErrorPayload)
            }
        }
        operation.fetchRecordZoneChangesCompletionBlock = { error in
            collector.finish(error)
        }

        database.add(operation)
        let timedOut = ckWaitCancellingOnTimeout(operation, collector.finished, timeoutSeconds: timeoutSeconds)
        let (zones, encodeError, operationError) = collector.snapshot()
        if let encodeError {
            throw ckBridgeNSError(
                code: CKR_FAILURE,
                message: "CKFetchRecordZoneChangesOperation returned a changed record the bridge cannot encode, so the batch is rejected instead of silently dropping it: \(encodeError.localizedDescription)"
            )
        }
        let reportedError = timedOut ? ckTimeoutNSError("CKFetchRecordZoneChangesOperation", timeoutSeconds: timeoutSeconds) : operationError
        let result = CKFetchRecordZoneChangesResultPayload(zones: zones, operationError: reportedError.map(ckErrorPayload))
        outJSON?.pointee = ckCString(try ckEncodeJSON(result))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_database_execute_fetch_web_auth_token_sync")
public func ckDatabaseExecuteFetchWebAuthTokenSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ apiToken: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        guard let apiToken else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Missing API token")
        }
        let operation = CKFetchWebAuthTokenOperation(apiToken: String(cString: apiToken))
        let results = CKItemResults<Int, String>()
        operation.fetchWebAuthTokenResultBlock = { operationResult in
            switch operationResult {
            case .success(let webAuthToken):
                results.append(0, .success(webAuthToken))
                results.finish(.success(()))
            case .failure(let error):
                results.finish(.failure(error))
            }
        }
        let outcome = ckRunOperation(operation, on: database, results: results, timeoutSeconds: timeoutSeconds)
        let webAuthToken = try outcome.single(label: "CKFetchWebAuthTokenOperation", timeoutSeconds: timeoutSeconds, writes: false)
        outJSON?.pointee = ckCString(try ckEncodeJSON(webAuthToken))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}
