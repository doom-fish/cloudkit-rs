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
