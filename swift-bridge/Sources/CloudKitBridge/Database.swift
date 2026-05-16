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

@_cdecl("ck_database_fetch_query_results_sync")
public func ckDatabaseFetchQueryResultsSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ queryJSON: UnsafePointer<CChar>?,
    _ zoneIDJSON: UnsafePointer<CChar>?,
    _ desiredKeysJSON: UnsafePointer<CChar>?,
    _ resultsLimit: Int32,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let query = ckDecodeQuery(try ckDecodeJSON(queryJSON, as: CKQueryPayload.self))
        let zoneID = try zoneIDJSON.map { try ckDecodeJSON($0, as: CKRecordZoneIDPayload.self) }.map(ckDecodeZoneID)
        let desiredKeys = try desiredKeysJSON.map { try ckDecodeJSON($0, as: [String].self) }
        let limit = resultsLimit > 0 ? Int(resultsLimit) : CKQueryOperation.maximumResults
        let results = try ckAwait(label: "CloudKit fetch query results") { completion in
            database.fetch(withQuery: query, inZoneWith: zoneID, desiredKeys: desiredKeys, resultsLimit: limit) { result in
                switch result {
                case .success(let output):
                    completion(output, nil)
                case .failure(let error):
                    completion(nil, error as NSError)
                }
            }
        }

        let records = try results.matchResults.compactMap { try? $0.1.get() }.map(ckEncodeRecord)
        let matches = try results.matchResults.map { recordID, result in
            switch result {
            case .success(let record):
                return CKQueryMatchResultPayload(recordID: ckEncodeRecordID(recordID), record: try ckEncodeRecord(record), error: nil)
            case .failure(let error):
                return CKQueryMatchResultPayload(recordID: ckEncodeRecordID(recordID), record: nil, error: ckErrorPayload(from: error as NSError))
            }
        }
        let payload = CKFetchedQueryResultsPayload(
            records: records,
            matches: matches,
            cursor: results.queryCursor.map(ckEncodeQueryCursor),
            operationError: nil
        )
        outJSON?.pointee = ckCString(try ckEncodeJSON(payload))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_fetch_all_record_zones_sync")
public func ckDatabaseFetchAllRecordZonesSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let zones = try ckAwait(label: "CloudKit fetch all record zones") { completion in
            database.fetchAllRecordZones { zones, error in
                completion(zones, error as NSError?)
            }
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(zones.map(ckEncodeZone)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_fetch_record_zone_sync")
public func ckDatabaseFetchRecordZoneSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ zoneIDJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let zoneIDPayload = try ckDecodeJSON(zoneIDJSON, as: CKRecordZoneIDPayload.self)
        let zone = try ckAwait(label: "CloudKit fetch record zone") { completion in
            database.fetch(withRecordZoneID: ckDecodeZoneID(zoneIDPayload)) { zone, error in
                completion(zone, error as NSError?)
            }
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeZone(zone)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_save_record_zone_sync")
public func ckDatabaseSaveRecordZoneSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ zoneJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let zonePayload = try ckDecodeJSON(zoneJSON, as: CKRecordZonePayload.self)
        let zone = try ckAwait(label: "CloudKit save record zone") { completion in
            database.save(ckDecodeZone(zonePayload)) { zone, error in
                completion(zone, error as NSError?)
            }
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeZone(zone)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_delete_record_zone_sync")
public func ckDatabaseDeleteRecordZoneSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ zoneIDJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let zoneIDPayload = try ckDecodeJSON(zoneIDJSON, as: CKRecordZoneIDPayload.self)
        let deletedZoneID = try ckAwait(label: "CloudKit delete record zone") { completion in
            database.delete(withRecordZoneID: ckDecodeZoneID(zoneIDPayload)) { zoneID, error in
                completion(zoneID, error as NSError?)
            }
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeZoneID(deletedZoneID)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_fetch_subscription_sync")
public func ckDatabaseFetchSubscriptionSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ subscriptionID: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard let subscriptionID else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Missing subscription ID")
        }
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let subscription = try ckAwait(label: "CloudKit fetch subscription") { completion in
            database.fetch(withSubscriptionID: String(cString: subscriptionID)) { subscription, error in
                completion(subscription, error as NSError?)
            }
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeSubscription(subscription)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_fetch_all_subscriptions_sync")
public func ckDatabaseFetchAllSubscriptionsSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let subscriptions = try ckAwait(label: "CloudKit fetch all subscriptions") { completion in
            database.fetchAllSubscriptions { subscriptions, error in
                completion(subscriptions, error as NSError?)
            }
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(subscriptions.map(ckEncodeSubscription)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_save_subscription_sync")
public func ckDatabaseSaveSubscriptionSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ subscriptionJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let subscriptionPayload = try ckDecodeJSON(subscriptionJSON, as: CKSubscriptionPayload.self)
        let savedSubscription = try ckAwait(label: "CloudKit save subscription") { completion in
            database.save(ckDecodeSubscription(subscriptionPayload)) { subscription, error in
                completion(subscription, error as NSError?)
            }
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeSubscription(savedSubscription)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_database_delete_subscription_sync")
public func ckDatabaseDeleteSubscriptionSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ databaseScope: Int32,
    _ subscriptionID: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard let subscriptionID else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Missing subscription ID")
        }
        let database = try ckMakeDatabase(containerIdentifier: containerIdentifier, scopeRaw: databaseScope)
        let deletedSubscriptionID = try ckAwait(label: "CloudKit delete subscription") { completion in
            database.delete(withSubscriptionID: String(cString: subscriptionID)) { subscriptionID, error in
                completion(subscriptionID, error as NSError?)
            }
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(deletedSubscriptionID))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}
