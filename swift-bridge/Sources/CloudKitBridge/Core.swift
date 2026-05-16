import CloudKit
import Foundation
import Security

let CKR_OK: Int32 = 0
let CKR_INVALID_ARGUMENT: Int32 = -1
let CKR_FAILURE: Int32 = -2
let CKR_TIMED_OUT: Int32 = -3
let CKR_DEFAULT_CONTAINER_UNAVAILABLE: Int32 = -4
let CKR_BRIDGE_ERROR_DOMAIN = "CloudKitBridge"
private let ckICloudContainerIdentifiersEntitlement = "com.apple.developer.icloud-container-identifiers" as CFString

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

func ckTimeoutNSError(_ label: String) -> NSError {
    ckBridgeNSError(code: CKR_TIMED_OUT, message: "Timed out waiting for \(label)")
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

func ckErrorPayload(from error: NSError) -> CKErrorPayload {
    CKErrorPayload(
        domain: error.domain,
        code: error.code,
        message: error.localizedDescription,
        retryAfterSeconds: (error.userInfo[CKErrorRetryAfterKey] as? NSNumber)?.doubleValue
    )
}

func ckWriteError(_ error: NSError, to outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?) {
    guard let outError else { return }
    let payload = ckErrorPayload(from: error)
    let json = (try? ckEncodeJSON(payload)) ?? "{\"domain\":\"\(CKR_BRIDGE_ERROR_DOMAIN)\",\"code\":-2,\"message\":\"Unknown CloudKit bridge error\",\"retryAfterSeconds\":null}"
    outError.pointee = ckCString(json)
}

func ckDefaultContainerIdentifier() -> String? {
    guard let task = SecTaskCreateFromSelf(nil),
          let value = SecTaskCopyValueForEntitlement(task, ckICloudContainerIdentifiersEntitlement, nil) else {
        return nil
    }

    if let identifiers = value as? [String] {
        return identifiers.first { !$0.isEmpty }
    }
    if let identifiers = value as? NSArray {
        return identifiers.compactMap { $0 as? String }.first { !$0.isEmpty }
    }
    return nil
}

func ckMakeContainer(_ identifier: UnsafePointer<CChar>?) throws -> CKContainer {
    if let identifier {
        let identifier = String(cString: identifier)
        guard !identifier.isEmpty else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "CloudKit container identifier must not be empty")
        }
        return CKContainer(identifier: identifier)
    }

    guard let identifier = ckDefaultContainerIdentifier() else {
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

func ckAwait<T>(
    label: String,
    timeoutSeconds: TimeInterval = 30,
    _ body: (@escaping (T?, NSError?) -> Void) -> Void
) throws -> T {
    let semaphore = DispatchSemaphore(value: 0)
    let holder = CKResultHolder<T>()
    body { value, error in
        holder.value = value
        holder.error = error
        semaphore.signal()
    }

    if semaphore.wait(timeout: .now() + timeoutSeconds) == .timedOut {
        throw ckTimeoutNSError(label)
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
        let payload = (try? ckEncodeJSON(ckErrorPayload(from: error))) ?? "{\"domain\":\"\(error.domain)\",\"code\":\(error.code),\"message\":\"\(error.localizedDescription)\",\"retryAfterSeconds\":null}"
        payload.withCString { callback(refcon, nil, $0) }
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
        if let error {
            let payload = (try? ckEncodeJSON(ckErrorPayload(from: error))) ?? "{\"domain\":\"\(error.domain)\",\"code\":\(error.code),\"message\":\"\(error.localizedDescription)\",\"retryAfterSeconds\":null}"
            payload.withCString { callback(refcon, Int32(status.rawValue), $0) }
            return
        }
        callback(refcon, Int32(status.rawValue), nil)
    }
}
