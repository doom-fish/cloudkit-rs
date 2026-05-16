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
