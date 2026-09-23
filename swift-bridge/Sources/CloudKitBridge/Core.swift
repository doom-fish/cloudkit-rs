import CloudKit
import CloudKitObjCBridge
import Foundation
import Security

let CKR_OK: Int32 = 0
let CKR_INVALID_ARGUMENT: Int32 = -1
let CKR_FAILURE: Int32 = -2
let CKR_TIMED_OUT: Int32 = -3
let CKR_DEFAULT_CONTAINER_UNAVAILABLE: Int32 = -4
let CKR_CONTAINER_NOT_ENTITLED: Int32 = -5
let CKR_BRIDGE_ERROR_DOMAIN = "CloudKitBridge"
private let ckICloudContainerIdentifiersEntitlement = "com.apple.developer.icloud-container-identifiers" as CFString
private let ckFallbackErrorJSON = "{\"domain\":\"CloudKitBridge\",\"code\":-2,\"message\":\"CloudKit bridge could not encode an error payload\",\"retryAfterSeconds\":null}"

@_cdecl("ck_string_free")
public func ckStringFree(_ string: UnsafeMutablePointer<CChar>?) {
    free(string)
}

func ckCString(_ string: String) -> UnsafeMutablePointer<CChar>? {
    string.withCString { strdup($0) }
}

func ckRetain(_ object: some AnyObject) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(object).toOpaque()
}

func ckUnretained<T: AnyObject>(_ ptr: UnsafeMutableRawPointer, as type: T.Type = T.self) -> T {
    Unmanaged<T>.fromOpaque(ptr).takeUnretainedValue()
}

func ckRelease(_ ptr: UnsafeMutableRawPointer) {
    Unmanaged<AnyObject>.fromOpaque(ptr).release()
}

final class CKResultHolder<T>: @unchecked Sendable {
    private let lock = NSLock()
    private var _value: T?
    private var _error: NSError?

    var value: T? {
        get { lock.lock(); defer { lock.unlock() }; return _value }
        set { lock.lock(); defer { lock.unlock() }; _value = newValue }
    }

    var error: NSError? {
        get { lock.lock(); defer { lock.unlock() }; return _error }
        set { lock.lock(); defer { lock.unlock() }; _error = newValue }
    }
}

struct CKErrorPayload: Codable {
    var domain: String
    var code: Int
    var message: String
    var retryAfterSeconds: Double?
}

func ckBridgeNSError(code: Int32, message: String) -> NSError {
    NSError(domain: CKR_BRIDGE_ERROR_DOMAIN, code: Int(code), userInfo: [NSLocalizedDescriptionKey: message])
}

func ckInvalidArgument(_ message: String) -> NSError {
    ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: message)
}

func ckTimeoutNSError(_ label: String, timeoutSeconds: Double) -> NSError {
    ckBridgeNSError(code: CKR_TIMED_OUT, message: "Timed out after \(timeoutSeconds) s waiting for \(label)")
}

func ckCancelledAfterTimeoutNSError(_ label: String, timeoutSeconds: Double, underlying: NSError?) -> NSError {
    var message = "\(label) did not finish within \(timeoutSeconds) s and was cancelled; CloudKit may already have applied part of it, so fetch the affected items before retrying"
    if let underlying {
        message += " (\(underlying.domain) \(underlying.code): \(underlying.localizedDescription))"
    }
    return ckBridgeNSError(code: CKR_TIMED_OUT, message: message)
}

func ckFailureStatus(_ error: NSError) -> Int32 {
    guard let code = Int32(exactly: error.code), code != CKR_OK else {
        return CKR_FAILURE
    }
    return code
}

func ckObjCResult<T>(_ value: T?, _ error: NSError?, _ label: String) throws -> T {
    if let value {
        return value
    }
    throw error ?? ckInvalidArgument("\(label) failed")
}

func ckEncodeJSON<T: Encodable>(_ value: T) throws -> String {
    let data = try JSONEncoder().encode(value)
    guard let string = String(data: data, encoding: .utf8) else {
        throw ckBridgeNSError(code: CKR_FAILURE, message: "Failed to encode JSON as UTF-8")
    }
    return string
}

func ckDecodeJSON<T: Decodable>(_ cString: UnsafePointer<CChar>?, as type: T.Type) throws -> T {
    guard let cString else {
        throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Missing JSON payload")
    }
    let data = Data(String(cString: cString).utf8)
    do {
        return try JSONDecoder().decode(T.self, from: data)
    } catch {
        throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Invalid JSON payload: \(error.localizedDescription)")
    }
}

func ckArchiveSecureCoding(_ object: NSSecureCoding) throws -> [UInt8] {
    do {
        return [UInt8](try NSKeyedArchiver.archivedData(withRootObject: object, requiringSecureCoding: true))
    } catch let error as NSError {
        throw ckBridgeNSError(code: CKR_FAILURE, message: "Failed to archive secure coding object: \(error.localizedDescription)")
    }
}

func ckDecodeSecureCodingObject<T: NSObject & NSSecureCoding>(_ archivedData: [UInt8], as type: T.Type) throws -> T {
    do {
        guard let object = try NSKeyedUnarchiver.unarchivedObject(ofClass: type, from: Data(archivedData)) else {
            throw ckBridgeNSError(code: CKR_FAILURE, message: "Missing archived \(String(describing: type)) object")
        }
        return object
    } catch let error as NSError {
        if error.domain == CKR_BRIDGE_ERROR_DOMAIN {
            throw error
        }
        throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Failed to decode archived \(String(describing: type)) object: \(error.localizedDescription)")
    }
}

func ckErrorPayload(from error: NSError) -> CKErrorPayload {
    let retryAfterSeconds = (error.userInfo[CKErrorRetryAfterKey] as? NSNumber)?.doubleValue
    return CKErrorPayload(
        domain: error.domain,
        code: error.code,
        message: error.localizedDescription,
        retryAfterSeconds: retryAfterSeconds.flatMap { $0.isFinite ? $0 : nil }
    )
}

func ckErrorJSON(_ error: NSError) -> String {
    (try? ckEncodeJSON(ckErrorPayload(from: error))) ?? ckFallbackErrorJSON
}

func ckWriteError(_ error: NSError, to outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?) {
    guard let outError else { return }
    outError.pointee = ckCString(ckErrorJSON(error))
}

func ckEntitledContainerIdentifiers() -> [String] {
    guard let task = SecTaskCreateFromSelf(nil),
          let value = SecTaskCopyValueForEntitlement(task, ckICloudContainerIdentifiersEntitlement, nil) else {
        return []
    }

    if let identifiers = value as? [String] {
        return identifiers.filter { !$0.isEmpty }
    }
    if let identifiers = value as? NSArray {
        return identifiers.compactMap { $0 as? String }.filter { !$0.isEmpty }
    }
    return []
}

func ckMakeContainer(_ identifier: UnsafePointer<CChar>?) throws -> CKContainer {
    let entitledIdentifiers = ckEntitledContainerIdentifiers()
    if let identifier {
        let identifier = String(cString: identifier)
        guard !identifier.isEmpty else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "CloudKit container identifier must not be empty")
        }
        guard entitledIdentifiers.contains(identifier) else {
            throw ckBridgeNSError(
                code: CKR_CONTAINER_NOT_ENTITLED,
                message: "CloudKit container \(identifier) is not listed in this process's com.apple.developer.icloud-container-identifiers entitlement. CloudKit terminates processes that open unentitled containers, so the bridge refuses to open it."
            )
        }
        return CKContainer(identifier: identifier)
    }

    guard let identifier = entitledIdentifiers.first else {
        throw ckBridgeNSError(
            code: CKR_DEFAULT_CONTAINER_UNAVAILABLE,
            message: "CloudKit default container is unavailable for this process because no iCloud container entitlement was found. Use CKContainer::container(\"iCloud.<container-id>\") or run inside a signed app bundle with CloudKit entitlements."
        )
    }
    return CKContainer(identifier: identifier)
}

func ckMakeDatabase(containerIdentifier: UnsafePointer<CChar>?, scopeRaw: Int32) throws -> CKDatabase {
    let container = try ckMakeContainer(containerIdentifier)
    guard let scope = CKDatabase.Scope(rawValue: Int(scopeRaw)) else {
        throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Invalid database scope: \(scopeRaw)")
    }
    return container.database(with: scope)
}

func ckWait(_ semaphore: DispatchSemaphore, timeoutSeconds: Double) -> Bool {
    guard timeoutSeconds >= 0 else {
        semaphore.wait()
        return true
    }
    return semaphore.wait(timeout: .now() + timeoutSeconds) == .success
}

func ckWaitCancellingOnTimeout(_ operation: Operation, _ finished: DispatchSemaphore, timeoutSeconds: Double) -> Bool {
    if ckWait(finished, timeoutSeconds: timeoutSeconds) {
        return false
    }
    operation.cancel()
    finished.wait()
    return true
}

func ckAwait<T>(
    label: String,
    timeoutSeconds: Double,
    _ body: (@escaping (T?, NSError?) -> Void) -> Void
) throws -> T {
    let semaphore = DispatchSemaphore(value: 0)
    let holder = CKResultHolder<T>()
    body { value, error in
        holder.value = value
        holder.error = error
        semaphore.signal()
    }

    if !ckWait(semaphore, timeoutSeconds: timeoutSeconds) {
        throw ckTimeoutNSError(label, timeoutSeconds: timeoutSeconds)
    }
    if let error = holder.error {
        throw error
    }
    guard let value = holder.value else {
        throw ckBridgeNSError(code: CKR_FAILURE, message: "Missing result for \(label)")
    }
    return value
}

public typealias CKJSONStringCallback = @convention(c) (
    UnsafeMutableRawPointer?, UnsafePointer<CChar>?, UnsafePointer<CChar>?
) -> Void

public typealias CKAccountStatusCallback = @convention(c) (
    UnsafeMutableRawPointer?, Int32, UnsafePointer<CChar>?
) -> Void

final class CKJSONStringCallbackBox: @unchecked Sendable {
    let callback: CKJSONStringCallback
    let refcon: UnsafeMutableRawPointer?

    init(callback: @escaping CKJSONStringCallback, refcon: UnsafeMutableRawPointer?) {
        self.callback = callback
        self.refcon = refcon
    }

    func succeed(json: String) {
        json.withCString { callback(refcon, $0, nil) }
    }

    func fail(error: NSError) {
        ckErrorJSON(error).withCString { callback(refcon, nil, $0) }
    }
}

final class CKAccountStatusCallbackBox: @unchecked Sendable {
    let callback: CKAccountStatusCallback
    let refcon: UnsafeMutableRawPointer?

    init(callback: @escaping CKAccountStatusCallback, refcon: UnsafeMutableRawPointer?) {
        self.callback = callback
        self.refcon = refcon
    }

    func complete(status: CKAccountStatus, error: NSError?) {
        let rawStatus = Int32(clamping: status.rawValue)
        if let error {
            ckErrorJSON(error).withCString { callback(refcon, rawStatus, $0) }
            return
        }
        callback(refcon, rawStatus, nil)
    }
}
