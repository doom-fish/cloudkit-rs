import CloudKit
import Foundation

@_cdecl("ck_database_execute_modify_records_sync")
public func ckDatabaseExecuteModifyRecordsSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ operationJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let payload = try ckDecodeJSON(operationJSON, as: CKModifyRecordsOperationPayload.self)
        let operation = CKModifyRecordsOperation(
            recordsToSave: try payload.recordsToSave.map(ckDecodeRecord),
            recordIDsToDelete: payload.recordIDsToDelete.map(ckDecodeRecordID)
        )
        if let savePolicy = CKModifyRecordsOperation.RecordSavePolicy(rawValue: Int(payload.savePolicy)) {
            operation.savePolicy = savePolicy
        }
        operation.isAtomic = payload.atomic

        var savedRecords: [CKRecordPayload] = []
        var deletedRecordIDs: [CKRecordIDPayload] = []
        var saveResults: [CKRecordSaveResultPayload] = []
        var deleteResults: [CKRecordDeleteResultPayload] = []
        let completion = CKResultHolder<CKModifyRecordsResultPayload>()
        let semaphore = DispatchSemaphore(value: 0)

        operation.perRecordSaveBlock = { recordID, saveResult in
            switch saveResult {
            case .success(let record):
                let payload = try? ckEncodeRecord(record)
                if let payload { savedRecords.append(payload) }
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
        operation.perRecordDeleteBlock = { recordID, deleteResult in
            switch deleteResult {
            case .success:
                deletedRecordIDs.append(ckEncodeRecordID(recordID))
                deleteResults.append(
                    CKRecordDeleteResultPayload(recordID: ckEncodeRecordID(recordID), error: nil)
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
        operation.modifyRecordsResultBlock = { operationResult in
            let operationError: CKErrorPayload?
            switch operationResult {
            case .success:
                operationError = nil
            case .failure(let error):
                operationError = ckErrorPayload(from: error as NSError)
            }
            completion.value = CKModifyRecordsResultPayload(
                savedRecords: savedRecords,
                deletedRecordIDs: deletedRecordIDs,
                saveResults: saveResults,
                deleteResults: deleteResults,
                operationError: operationError
            )
            semaphore.signal()
        }

        database.add(operation)
        if semaphore.wait(timeout: .now() + 30) == .timedOut {
            throw ckTimeoutNSError("CloudKit CKModifyRecordsOperation")
        }
        guard let result = completion.value else {
            throw ckBridgeNSError(code: CKR_FAILURE, message: "CKModifyRecordsOperation completed without a result payload")
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(result))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_execute_query_operation_sync")
public func ckDatabaseExecuteQueryOperationSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ operationJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let payload = try ckDecodeJSON(operationJSON, as: CKQueryOperationPayload.self)
        let operation = CKQueryOperation(query: ckDecodeQuery(payload.query))
        operation.zoneID = payload.zoneID.map(ckDecodeZoneID)
        operation.desiredKeys = payload.desiredKeys
        if let resultsLimit = payload.resultsLimit {
            operation.resultsLimit = resultsLimit
        }

        var records: [CKRecordPayload] = []
        var matches: [CKQueryMatchResultPayload] = []
        let completion = CKResultHolder<CKQueryOperationResultPayload>()
        let semaphore = DispatchSemaphore(value: 0)

        operation.recordMatchedBlock = { recordID, recordResult in
            switch recordResult {
            case .success(let record):
                let payload = try? ckEncodeRecord(record)
                if let payload { records.append(payload) }
                matches.append(
                    CKQueryMatchResultPayload(
                        recordID: ckEncodeRecordID(recordID),
                        record: payload,
                        error: nil
                    )
                )
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
        operation.queryResultBlock = { operationResult in
            let cursorReturned: Bool
            let operationError: CKErrorPayload?
            switch operationResult {
            case .success(let cursor):
                cursorReturned = cursor != nil
                operationError = nil
            case .failure(let error):
                cursorReturned = false
                operationError = ckErrorPayload(from: error as NSError)
            }
            completion.value = CKQueryOperationResultPayload(
                records: records,
                matches: matches,
                cursorReturned: cursorReturned,
                operationError: operationError
            )
            semaphore.signal()
        }

        database.add(operation)
        if semaphore.wait(timeout: .now() + 30) == .timedOut {
            throw ckTimeoutNSError("CloudKit CKQueryOperation")
        }
        guard let result = completion.value else {
            throw ckBridgeNSError(code: CKR_FAILURE, message: "CKQueryOperation completed without a result payload")
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(result))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_execute_fetch_records_sync")
public func ckDatabaseExecuteFetchRecordsSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ operationJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let payload = try ckDecodeJSON(operationJSON, as: CKFetchRecordsOperationPayload.self)
        let operation = CKFetchRecordsOperation(recordIDs: payload.recordIDs.map(ckDecodeRecordID))
        operation.desiredKeys = payload.desiredKeys

        var records: [CKRecordPayload] = []
        var results: [CKRecordResultPayload] = []
        let completion = CKResultHolder<CKFetchRecordsResultPayload>()
        let semaphore = DispatchSemaphore(value: 0)

        operation.perRecordCompletionBlock = { record, recordID, error in
            guard let recordID else { return }
            if let error = error as NSError? {
                results.append(
                    CKRecordResultPayload(
                        recordID: ckEncodeRecordID(recordID),
                        record: nil,
                        error: ckErrorPayload(from: error)
                    )
                )
                return
            }
            let payload = record.flatMap { try? ckEncodeRecord($0) }
            if let payload { records.append(payload) }
            results.append(
                CKRecordResultPayload(
                    recordID: ckEncodeRecordID(recordID),
                    record: payload,
                    error: nil
                )
            )
        }
        operation.fetchRecordsCompletionBlock = { _, error in
            completion.value = CKFetchRecordsResultPayload(
                records: records,
                results: results,
                operationError: (error as NSError?).map(ckErrorPayload)
            )
            semaphore.signal()
        }

        database.add(operation)
        if semaphore.wait(timeout: .now() + 30) == .timedOut {
            throw ckTimeoutNSError("CloudKit CKFetchRecordsOperation")
        }
        guard let result = completion.value else {
            throw ckBridgeNSError(code: CKR_FAILURE, message: "CKFetchRecordsOperation completed without a result payload")
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(result))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_execute_fetch_database_changes_sync")
public func ckDatabaseExecuteFetchDatabaseChangesSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ operationJSON: UnsafePointer<CChar>?,
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

        var changedZoneIDs: [CKRecordZoneIDPayload] = []
        var deletedZoneIDs: [CKRecordZoneIDPayload] = []
        var purgedZoneIDs: [CKRecordZoneIDPayload] = []
        var encryptedDataResetZoneIDs: [CKRecordZoneIDPayload] = []
        var updatedServerChangeTokens: [CKServerChangeTokenPayload] = []
        let completion = CKResultHolder<CKFetchDatabaseChangesResultPayload>()
        let semaphore = DispatchSemaphore(value: 0)

        operation.recordZoneWithIDChangedBlock = { zoneID in
            changedZoneIDs.append(ckEncodeZoneID(zoneID))
        }
        operation.recordZoneWithIDWasDeletedBlock = { zoneID in
            deletedZoneIDs.append(ckEncodeZoneID(zoneID))
        }
        operation.recordZoneWithIDWasPurgedBlock = { zoneID in
            purgedZoneIDs.append(ckEncodeZoneID(zoneID))
        }
        operation.recordZoneWithIDWasDeletedDueToUserEncryptedDataResetBlock = { zoneID in
            encryptedDataResetZoneIDs.append(ckEncodeZoneID(zoneID))
        }
        operation.changeTokenUpdatedBlock = { serverChangeToken in
            updatedServerChangeTokens.append(ckEncodeServerChangeToken(serverChangeToken))
        }
        operation.fetchDatabaseChangesCompletionBlock = { serverChangeToken, moreComing, error in
            completion.value = CKFetchDatabaseChangesResultPayload(
                changedZoneIDs: changedZoneIDs,
                deletedZoneIDs: deletedZoneIDs,
                purgedZoneIDs: purgedZoneIDs,
                encryptedDataResetZoneIDs: encryptedDataResetZoneIDs,
                updatedServerChangeTokens: updatedServerChangeTokens,
                serverChangeToken: serverChangeToken.map(ckEncodeServerChangeToken),
                moreComing: moreComing,
                operationError: (error as NSError?).map(ckErrorPayload)
            )
            semaphore.signal()
        }

        database.add(operation)
        if semaphore.wait(timeout: .now() + 30) == .timedOut {
            throw ckTimeoutNSError("CloudKit CKFetchDatabaseChangesOperation")
        }
        guard let result = completion.value else {
            throw ckBridgeNSError(code: CKR_FAILURE, message: "CKFetchDatabaseChangesOperation completed without a result payload")
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(result))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_execute_fetch_record_zone_changes_sync")
public func ckDatabaseExecuteFetchRecordZoneChangesSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ operationJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let payload = try ckDecodeJSON(operationJSON, as: CKFetchRecordZoneChangesOperationPayload.self)

        let zoneIDs = payload.zones.map { ckDecodeZoneID($0.zoneID) }
        let configurations = Dictionary(uniqueKeysWithValues: try payload.zones.map { entry in
            let configuration = CKFetchRecordZoneChangesOperation.ZoneConfiguration()
            configuration.previousServerChangeToken = try entry.configuration.previousServerChangeToken.map(ckDecodeServerChangeToken)
            if let resultsLimit = entry.configuration.resultsLimit {
                configuration.resultsLimit = resultsLimit
            }
            configuration.desiredKeys = entry.configuration.desiredKeys
            return (ckDecodeZoneID(entry.zoneID), configuration)
        })
        let operation = CKFetchRecordZoneChangesOperation(recordZoneIDs: zoneIDs, configurationsByRecordZoneID: configurations)
        operation.fetchAllChanges = payload.fetchAllChanges

        var zoneResults: [String: CKFetchRecordZoneResultPayload] = [:]
        let completion = CKResultHolder<CKFetchRecordZoneChangesResultPayload>()
        let semaphore = DispatchSemaphore(value: 0)

        func zoneKey(for zoneID: CKRecordZone.ID) -> String {
            "\(zoneID.zoneName)|\(zoneID.ownerName)"
        }

        func updateZoneResult(_ zoneID: CKRecordZone.ID, _ body: (inout CKFetchRecordZoneResultPayload) -> Void) {
            let key = zoneKey(for: zoneID)
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

        operation.recordChangedBlock = { record in
            updateZoneResult(record.recordID.zoneID) { result in
                if let payload = try? ckEncodeRecord(record) {
                    result.changedRecords.append(payload)
                }
            }
        }
        operation.recordWithIDWasDeletedBlock = { recordID, recordType in
            updateZoneResult(recordID.zoneID) { result in
                result.deletedRecords.append(
                    CKDeletedRecordPayload(recordID: ckEncodeRecordID(recordID), recordType: recordType)
                )
            }
        }
        operation.recordZoneChangeTokensUpdatedBlock = { zoneID, serverChangeToken, clientChangeTokenData in
            updateZoneResult(zoneID) { result in
                if let serverChangeToken {
                    result.updatedServerChangeTokens.append(ckEncodeServerChangeToken(serverChangeToken))
                }
                result.clientChangeTokenData = clientChangeTokenData.map { [UInt8]($0) }
            }
        }
        operation.recordZoneFetchCompletionBlock = { zoneID, serverChangeToken, clientChangeTokenData, moreComing, error in
            updateZoneResult(zoneID) { result in
                result.serverChangeToken = serverChangeToken.map(ckEncodeServerChangeToken)
                result.clientChangeTokenData = clientChangeTokenData.map { [UInt8]($0) }
                result.moreComing = moreComing
                result.zoneError = (error as NSError?).map(ckErrorPayload)
            }
        }
        operation.fetchRecordZoneChangesCompletionBlock = { error in
            let sortedZones = zoneResults.values.sorted {
                ($0.zoneID.zoneName, $0.zoneID.ownerName) < ($1.zoneID.zoneName, $1.zoneID.ownerName)
            }
            completion.value = CKFetchRecordZoneChangesResultPayload(
                zones: sortedZones,
                operationError: (error as NSError?).map(ckErrorPayload)
            )
            semaphore.signal()
        }

        database.add(operation)
        if semaphore.wait(timeout: .now() + 30) == .timedOut {
            throw ckTimeoutNSError("CloudKit CKFetchRecordZoneChangesOperation")
        }
        guard let result = completion.value else {
            throw ckBridgeNSError(code: CKR_FAILURE, message: "CKFetchRecordZoneChangesOperation completed without a result payload")
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(result))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}
