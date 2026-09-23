import CloudKit
import Foundation

@_cdecl("ck_container_account_status_sync")
public func ckContainerAccountStatusSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outStatus: UnsafeMutablePointer<Int32>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let container = try ckMakeContainer(containerIdentifier)
        let status = try ckAwait(label: "CloudKit account status", timeoutSeconds: timeoutSeconds) { completion in
            container.accountStatus { status, error in
                completion(status, error as NSError?)
            }
        }
        outStatus?.pointee = Int32(clamping: status.rawValue)
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
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
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let container = try ckMakeContainer(containerIdentifier)
        let recordID = try ckAwait(label: "CloudKit user record ID", timeoutSeconds: timeoutSeconds) { completion in
            container.fetchUserRecordID { recordID, error in
                completion(recordID, error as NSError?)
            }
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeRecordID(recordID)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
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
            defer { ckRelease(boxPtr) }
            if let error = error as NSError? {
                box.fail(error: error)
            } else if let recordID {
                do {
                    box.succeed(json: try ckEncodeJSON(ckEncodeRecordID(recordID)))
                } catch let error as NSError {
                    box.fail(error: error)
                }
            } else {
                box.fail(error: ckBridgeNSError(code: CKR_FAILURE, message: "CloudKit returned a nil user record ID"))
            }
        }
    } catch let error as NSError {
        let box: CKJSONStringCallbackBox = ckUnretained(boxPtr, as: CKJSONStringCallbackBox.self)
        box.fail(error: error)
        ckRelease(boxPtr)
    }
}

private func ckAwaitUserIdentity(
    container: CKContainer,
    lookupInfo: CKUserIdentityLookupInfoPayload,
    timeoutSeconds: Double
) throws -> CKUserIdentity {
    let label = "CloudKit discover user identity"
    if let emailAddress = lookupInfo.emailAddress {
        return try ckAwait(label: label, timeoutSeconds: timeoutSeconds) { completion in
            container.discoverUserIdentity(withEmailAddress: emailAddress) { identity, error in
                completion(identity, error as NSError?)
            }
        }
    }
    if let phoneNumber = lookupInfo.phoneNumber {
        return try ckAwait(label: label, timeoutSeconds: timeoutSeconds) { completion in
            container.discoverUserIdentity(withPhoneNumber: phoneNumber) { identity, error in
                completion(identity, error as NSError?)
            }
        }
    }
    if let userRecordID = lookupInfo.userRecordID {
        let recordID = try ckDecodeRecordID(userRecordID)
        return try ckAwait(label: label, timeoutSeconds: timeoutSeconds) { completion in
            container.discoverUserIdentity(withUserRecordID: recordID) { identity, error in
                completion(identity, error as NSError?)
            }
        }
    }
    throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "User identity lookup info must include an email address, phone number, or user record ID")
}

private func ckAwaitShareParticipant(
    container: CKContainer,
    lookupInfo: CKUserIdentityLookupInfoPayload,
    timeoutSeconds: Double
) throws -> CKShare.Participant {
    let results = CKItemResults<CKUserIdentity.LookupInfo, CKShare.Participant>()
    let operation = CKFetchShareParticipantsOperation(userIdentityLookupInfos: [try ckDecodeLookupInfo(lookupInfo)])
    operation.qualityOfService = .userInitiated
    operation.perShareParticipantResultBlock = { lookupInfo, participantResult in
        results.append(lookupInfo, participantResult)
    }
    operation.fetchShareParticipantsResultBlock = { operationResult in
        results.finish(operationResult)
    }
    container.add(operation)
    let timedOut = ckWaitCancellingOnTimeout(operation, results.finished, timeoutSeconds: timeoutSeconds)
    let outcome = CKOperationOutcome(results: results.results, operationError: results.operationError, timedOut: timedOut)
    return try outcome.single(label: "CloudKit fetch share participant", timeoutSeconds: timeoutSeconds, writes: false)
}

@_cdecl("ck_container_discover_user_identity_sync")
public func ckContainerDiscoverUserIdentitySync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ lookupInfoJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let container = try ckMakeContainer(containerIdentifier)
        let lookupInfo = try ckDecodeJSON(lookupInfoJSON, as: CKUserIdentityLookupInfoPayload.self)
        let identity = try ckAwaitUserIdentity(container: container, lookupInfo: lookupInfo, timeoutSeconds: timeoutSeconds)
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeUserIdentity(identity)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_container_fetch_share_participant_sync")
public func ckContainerFetchShareParticipantSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ lookupInfoJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let container = try ckMakeContainer(containerIdentifier)
        let lookupInfo = try ckDecodeJSON(lookupInfoJSON, as: CKUserIdentityLookupInfoPayload.self)
        let participant = try ckAwaitShareParticipant(container: container, lookupInfo: lookupInfo, timeoutSeconds: timeoutSeconds)
        outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeShareParticipant(participant)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}
