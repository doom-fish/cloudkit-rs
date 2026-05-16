import CloudKit
import Foundation

@_cdecl("ck_database_save_record_sync")
public func ckDatabaseSaveRecordSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ recordJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let payload = try ckDecodeJSON(recordJSON, as: CKRecordPayload.self)
        let record = try ckDecodeRecord(payload)
        let savedRecord = try ckAwait(label: "CloudKit saveRecord") { completion in
            database.save(record) { savedRecord, error in
                completion(savedRecord, error as NSError?)
            }
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(try ckEncodeRecord(savedRecord)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_fetch_record_sync")
public func ckDatabaseFetchRecordSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ recordIDJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let recordIDPayload = try ckDecodeJSON(recordIDJSON, as: CKRecordIDPayload.self)
        let fetchedRecord = try ckAwait(label: "CloudKit fetchRecord") { completion in
            database.fetch(withRecordID: ckDecodeRecordID(recordIDPayload)) { record, error in
                completion(record, error as NSError?)
            }
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(try ckEncodeRecord(fetchedRecord)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_delete_record_sync")
public func ckDatabaseDeleteRecordSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ recordIDJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let recordIDPayload = try ckDecodeJSON(recordIDJSON, as: CKRecordIDPayload.self)
        let deletedRecordID = try ckAwait(label: "CloudKit deleteRecord") { completion in
            database.delete(withRecordID: ckDecodeRecordID(recordIDPayload)) { recordID, error in
                completion(recordID, error as NSError?)
            }
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeRecordID(deletedRecordID)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_perform_query_sync")
public func ckDatabasePerformQuerySync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ queryJSON: UnsafePointer<CChar>?,
    _ zoneIDJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let query = ckDecodeQuery(try ckDecodeJSON(queryJSON, as: CKQueryPayload.self))
        let zoneID = try zoneIDJSON.map { try ckDecodeJSON($0, as: CKRecordZoneIDPayload.self) }.map(ckDecodeZoneID)
        let records = try ckAwait(label: "CloudKit performQuery") { completion in
            database.fetch(withQuery: query, inZoneWith: zoneID, desiredKeys: nil, resultsLimit: CKQueryOperation.maximumResults) { result in
                switch result {
                case .success(let output):
                    let records = output.matchResults.compactMap { try? $0.1.get() }
                    completion(records, nil)
                case .failure(let error):
                    completion(nil, error as NSError)
                }
            }
        }
        let payloads = try records.map(ckEncodeRecord)
        outJSON?.pointee = ckCString(try ckEncodeJSON(payloads))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
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
        let query = ckDecodeQuery(try ckDecodeJSON(queryJSON, as: CKQueryPayload.self))
        let zoneID = try zoneIDJSON.map { try ckDecodeJSON($0, as: CKRecordZoneIDPayload.self) }.map(ckDecodeZoneID)
        database.fetch(withQuery: query, inZoneWith: zoneID, desiredKeys: nil, resultsLimit: CKQueryOperation.maximumResults) { result in
            let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
            switch result {
            case .success(let output):
                let records = output.matchResults.compactMap { try? $0.1.get() }
                let payloads = (try? records.map(ckEncodeRecord)) ?? []
                let json = (try? ckEncodeJSON(payloads)) ?? "[]"
                box.succeed(json: json)
            case .failure(let error):
                box.fail(error: error as NSError)
            }
            ckRelease(boxPtr)
        }
    } catch let error as NSError {
        let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
        box.fail(error: error)
        ckRelease(boxPtr)
    }
}
