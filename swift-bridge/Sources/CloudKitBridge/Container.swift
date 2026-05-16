import CloudKit
import Foundation

@_cdecl("ck_record_create")
public func ckRecordCreate(
    _ recordType: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard let recordType else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Missing record type")
        }
        let record = CKRecord(recordType: String(cString: recordType))
        let payload = try ckEncodeRecord(record)
        outJSON?.pointee = ckCString(try ckEncodeJSON(payload))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_container_account_status_sync")
public func ckContainerAccountStatusSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ outStatus: UnsafeMutablePointer<Int32>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let container = try ckMakeContainer(containerIdentifier)
        let status = try ckAwait(label: "CloudKit account status") { completion in
            container.accountStatus { status, error in
                completion(status, error as NSError?)
            }
        }
        outStatus?.pointee = Int32(status.rawValue)
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_container_account_status_async")
public func ckContainerAccountStatusAsync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ callback: @escaping CKAccountStatusCallback,
    _ refcon: UnsafeMutableRawPointer?
) {
    let boxPtr = ckRetain(CKAccountStatusCallbackBox(callback: callback, refcon: refcon))
    do {
        let container = try ckMakeContainer(containerIdentifier)
        container.accountStatus { status, error in
            let box: CKAccountStatusCallbackBox = ckUnretained(boxPtr, as: CKAccountStatusCallbackBox.self)
            box.complete(status: status, error: error as NSError?)
            ckRelease(boxPtr)
        }
    } catch let error as NSError {
        let box: CKAccountStatusCallbackBox = ckUnretained(boxPtr, as: CKAccountStatusCallbackBox.self)
        box.complete(status: .couldNotDetermine, error: error)
        ckRelease(boxPtr)
    }
}

@_cdecl("ck_container_fetch_user_record_id_sync")
public func ckContainerFetchUserRecordIDSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let container = try ckMakeContainer(containerIdentifier)
        let recordID = try ckAwait(label: "CloudKit user record ID") { completion in
            container.fetchUserRecordID { recordID, error in
                completion(recordID, error as NSError?)
            }
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeRecordID(recordID)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_container_fetch_user_record_id_async")
public func ckContainerFetchUserRecordIDAsync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ callback: @escaping CKJSONStringCallback,
    _ refcon: UnsafeMutableRawPointer?
) {
    let boxPtr = ckRetain(CKJSONStringCallbackBox(callback: callback, refcon: refcon))
    do {
        let container = try ckMakeContainer(containerIdentifier)
        container.fetchUserRecordID { recordID, error in
            let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
            if let error = error as NSError? {
                box.fail(error: error)
            } else if let recordID {
                let json = (try? ckEncodeJSON(ckEncodeRecordID(recordID))) ?? "{}"
                box.succeed(json: json)
            } else {
                box.fail(error: ckBridgeNSError(code: CKR_FAILURE, message: "CloudKit returned a nil user record ID"))
            }
            ckRelease(boxPtr)
        }
    } catch let error as NSError {
        let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
        box.fail(error: error)
        ckRelease(boxPtr)
    }
}
