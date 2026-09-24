import CloudKit
import CloudKitObjCBridge
import Foundation

struct CKSharePayload: Codable {
    var shareRecord: CKRecordPayload
    var rootRecord: CKRecordPayload?
    var zoneID: CKRecordZoneIDPayload
    var publicPermission: Int32
    var url: String?
    var participants: [CKShareParticipantPayload]
    var owner: CKShareParticipantPayload?
    var currentUserParticipant: CKShareParticipantPayload?
    var title: String?
    var thumbnailImageData: [UInt8]?
    var shareType: String?
    var allowsAccessRequests: Bool?
    var isZoneWide: Bool
}

struct CKShareMetadataPayload: Codable {
    var archivedData: [UInt8]
    var containerIdentifier: String
    var share: CKSharePayload
    var hierarchicalRootRecordID: CKRecordIDPayload?
    var participantRole: Int?
    var participantStatus: Int
    var participantPermission: Int
    var ownerIdentity: CKUserIdentityPayload
    var rootRecord: CKRecordPayload?
    var participantType: Int?
    var rootRecordID: CKRecordIDPayload?
}

func ckEncodeShare(_ share: CKShare, rootRecord: CKRecordPayload?, isZoneWide: Bool) throws -> CKSharePayload {
    let title = share[CKShare.SystemFieldKey.title] as? String
    let thumbnailImageData = (share[CKShare.SystemFieldKey.thumbnailImageData] as? Data).map([UInt8].init)
    let shareType = share[CKShare.SystemFieldKey.shareType] as? String
    let allowsAccessRequests: Bool?
    if #available(macOS 26.0, *) {
        allowsAccessRequests = share.allowsAccessRequests
    } else {
        allowsAccessRequests = nil
    }
    return CKSharePayload(
        shareRecord: try ckEncodeRecord(share),
        rootRecord: rootRecord,
        zoneID: ckEncodeZoneID(share.recordID.zoneID),
        publicPermission: Int32(clamping: share.publicPermission.rawValue),
        url: share.url?.absoluteString,
        participants: share.participants.map(ckEncodeShareParticipant),
        owner: ckEncodeShareParticipant(share.owner),
        currentUserParticipant: share.currentUserParticipant.map(ckEncodeShareParticipant),
        title: title,
        thumbnailImageData: thumbnailImageData,
        shareType: shareType,
        allowsAccessRequests: allowsAccessRequests,
        isZoneWide: isZoneWide
    )
}

func ckSyncShareParticipants(from payload: CKSharePayload, into share: CKShare) throws {
    let desiredIDs = Set(payload.participants.map(\.participantID))
    let currentIDs = Set(share.participants.map(\.participantID))
    let removals = share.participants.filter { !desiredIDs.contains($0.participantID) }
    let additions = payload.participants.filter { !currentIDs.contains($0.participantID) }

    if removals.contains(where: { $0.participantID == share.owner.participantID }) {
        throw ckInvalidArgument("A CKShare's owner cannot be removed from the share")
    }
    guard !removals.isEmpty || !additions.isEmpty else {
        return
    }
    try ckRequireCloudKitEntitlements(for: "Changing a CKShare's participants")

    for participant in removals {
        var error: NSError?
        guard CKXShareRemoveParticipant(share, participant, &error) else {
            throw error ?? ckInvalidArgument("CKShare.removeParticipant(_:) rejected participant \(participant.participantID)")
        }
    }
    for participantPayload in additions {
        var error: NSError?
        guard CKXShareAddParticipant(share, try ckDecodeShareParticipant(participantPayload), &error) else {
            throw error ?? ckInvalidArgument("CKShare.addParticipant(_:) rejected participant \(participantPayload.participantID)")
        }
    }
}

func ckSharePublicPermission(_ rawValue: Int32) throws -> CKShare.ParticipantPermission {
    switch Int(rawValue) {
    case CKShare.ParticipantPermission.none.rawValue:
        return .none
    case CKShare.ParticipantPermission.readOnly.rawValue:
        return .readOnly
    case CKShare.ParticipantPermission.readWrite.rawValue:
        return .readWrite
    default:
        throw ckInvalidArgument("CKShare publicPermission must be none, readOnly or readWrite, not raw value \(rawValue)")
    }
}

func ckDecodeShare(_ payload: CKSharePayload) throws -> CKShare {
    let share: CKShare
    if !payload.shareRecord.encodedSystemFields.isEmpty {
        let data = Data(payload.shareRecord.encodedSystemFields)
        let unarchiver = try NSKeyedUnarchiver(forReadingFrom: data)
        unarchiver.requiresSecureCoding = true
        share = CKShare(coder: unarchiver)
        unarchiver.finishDecoding()
    } else if payload.isZoneWide {
        share = CKShare(recordZoneID: try ckDecodeZoneID(payload.zoneID))
    } else if let rootRecord = payload.rootRecord {
        share = CKShare(rootRecord: try ckDecodeRecord(rootRecord))
    } else {
        throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Share payload missing root record for non-zone-wide share")
    }

    if share.publicPermission.rawValue != Int(payload.publicPermission) {
        share.publicPermission = try ckSharePublicPermission(payload.publicPermission)
    }
    try ckApplyRecordPayload(payload.shareRecord, to: share)
    if let title = payload.title {
        share[CKShare.SystemFieldKey.title] = title as NSString
    }
    if let thumbnailImageData = payload.thumbnailImageData {
        share[CKShare.SystemFieldKey.thumbnailImageData] = NSData(data: Data(thumbnailImageData))
    }
    if let shareType = payload.shareType {
        share[CKShare.SystemFieldKey.shareType] = shareType as NSString
    }
    if let allowsAccessRequests = payload.allowsAccessRequests {
        guard #available(macOS 26.0, *) else {
            throw ckBridgeNSError(code: CKR_FAILURE, message: "CKShare.allowsAccessRequests requires macOS 26.0 or newer")
        }
        if share.allowsAccessRequests != allowsAccessRequests {
            try ckRequireCloudKitEntitlements(for: "Setting CKShare.allowsAccessRequests")
            var error: NSError?
            guard CKXShareSetAllowsAccessRequests(share, allowsAccessRequests, &error) else {
                throw error ?? ckInvalidArgument("CKShare.allowsAccessRequests rejected \(allowsAccessRequests)")
            }
        }
    }
    try ckSyncShareParticipants(from: payload, into: share)
    return share
}

func ckEncodeShareMetadata(_ metadata: CKShare.Metadata) throws -> CKShareMetadataPayload {
    let rootRecord = try metadata.rootRecord.map(ckEncodeRecord)
    return CKShareMetadataPayload(
        archivedData: try ckArchiveSecureCoding(metadata),
        containerIdentifier: metadata.containerIdentifier,
        share: try ckEncodeShare(
            metadata.share,
            rootRecord: rootRecord,
            isZoneWide: metadata.share.recordID.recordName == CKRecordNameZoneWideShare
        ),
        hierarchicalRootRecordID: metadata.hierarchicalRootRecordID.map(ckEncodeRecordID),
        participantRole: {
            if #available(macOS 10.14, *) {
                return metadata.participantRole.rawValue
            }
            return nil
        }(),
        participantStatus: metadata.participantStatus.rawValue,
        participantPermission: metadata.participantPermission.rawValue,
        ownerIdentity: ckEncodeUserIdentity(metadata.ownerIdentity),
        rootRecord: rootRecord,
        participantType: metadata.participantType.rawValue,
        rootRecordID: ckEncodeRecordID(metadata.rootRecordID)
    )
}

func ckDecodeShareMetadata(_ payload: CKShareMetadataPayload) throws -> CKShare.Metadata {
    try ckDecodeSecureCodingObject(payload.archivedData, as: CKShare.Metadata.self)
}

@_cdecl("ck_share_create_root_record")
public func ckShareCreateRootRecord(
    _ rootRecordJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let rootRecordPayload = try ckDecodeJSON(rootRecordJSON, as: CKRecordPayload.self)
        let share = CKShare(rootRecord: try ckDecodeRecord(rootRecordPayload))
        outJSON?.pointee = ckCString(try ckEncodeJSON(try ckEncodeShare(share, rootRecord: rootRecordPayload, isZoneWide: false)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_share_create_zone_wide")
public func ckShareCreateZoneWide(
    _ zoneIDJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let zoneIDPayload = try ckDecodeJSON(zoneIDJSON, as: CKRecordZoneIDPayload.self)
        let share = CKShare(recordZoneID: try ckDecodeZoneID(zoneIDPayload))
        outJSON?.pointee = ckCString(try ckEncodeJSON(try ckEncodeShare(share, rootRecord: nil, isZoneWide: true)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_share_normalize")
public func ckShareNormalize(
    _ shareJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let payload = try ckDecodeJSON(shareJSON, as: CKSharePayload.self)
        let share = try ckDecodeShare(payload)
        outJSON?.pointee = ckCString(try ckEncodeJSON(try ckEncodeShare(share, rootRecord: payload.rootRecord, isZoneWide: payload.isZoneWide)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_share_create_one_time_url_participant")
public func ckShareCreateOneTimeURLParticipant(
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        if #available(macOS 15.0, *) {
            outJSON?.pointee = ckCString(try ckEncodeJSON(ckEncodeShareParticipant(.oneTimeURLParticipant())))
            return CKR_OK
        }
        throw ckBridgeNSError(code: CKR_FAILURE, message: "One-time URL participants require macOS 15.0 or newer")
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_container_fetch_share_metadata_sync")
public func ckContainerFetchShareMetadataSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ shareURL: UnsafePointer<CChar>?,
    _ shouldFetchRootRecord: Bool,
    _ rootRecordDesiredKeysJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let container = try ckMakeContainer(containerIdentifier)
        guard let shareURL else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Missing share URL")
        }
        let desiredKeys = try rootRecordDesiredKeysJSON.map { try ckDecodeJSON($0, as: [String].self) }
        guard let url = URL(string: String(cString: shareURL)) else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Invalid share URL")
        }

        let results = CKItemResults<URL, CKShare.Metadata>()
        let operation = CKFetchShareMetadataOperation(shareURLs: [url])
        operation.shouldFetchRootRecord = shouldFetchRootRecord
        operation.rootRecordDesiredKeys = desiredKeys
        operation.perShareMetadataResultBlock = { shareURL, shareMetadataResult in
            results.append(shareURL, shareMetadataResult)
        }
        operation.fetchShareMetadataResultBlock = { operationResult in
            results.finish(operationResult)
        }

        container.add(operation)
        let timedOut = ckWaitCancellingOnTimeout(operation, results.finished, timeoutSeconds: timeoutSeconds)
        let outcome = CKOperationOutcome(results: results.results, operationError: results.operationError, timedOut: timedOut)
        let shareMetadata = try outcome.single(label: "CKFetchShareMetadataOperation", timeoutSeconds: timeoutSeconds, writes: false)
        outJSON?.pointee = ckCString(try ckEncodeJSON(try ckEncodeShareMetadata(shareMetadata)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_container_accept_share_metadata_sync")
public func ckContainerAcceptShareMetadataSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ shareMetadataJSON: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let container = try ckMakeContainer(containerIdentifier)
        let payload = try ckDecodeJSON(shareMetadataJSON, as: CKShareMetadataPayload.self)
        let shareMetadata = try ckDecodeShareMetadata(payload)

        let results = CKItemResults<CKShare.Metadata, CKShare>()
        let operation = CKAcceptSharesOperation(shareMetadatas: [shareMetadata])
        operation.qualityOfService = .userInitiated
        operation.perShareResultBlock = { metadata, shareResult in
            results.append(metadata, shareResult)
        }
        operation.acceptSharesResultBlock = { operationResult in
            results.finish(operationResult)
        }

        container.add(operation)
        let timedOut = ckWaitCancellingOnTimeout(operation, results.finished, timeoutSeconds: timeoutSeconds)
        let outcome = CKOperationOutcome(results: results.results, operationError: results.operationError, timedOut: timedOut)
        let acceptedShare = try outcome.single(label: "CKAcceptSharesOperation", timeoutSeconds: timeoutSeconds, writes: true)
        let acceptedPayload = try ckEncodeShare(
            acceptedShare,
            rootRecord: payload.rootRecord,
            isZoneWide: acceptedShare.recordID.recordName == CKRecordNameZoneWideShare
        )
        outJSON?.pointee = ckCString(try ckEncodeJSON(acceptedPayload))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}

@_cdecl("ck_container_request_share_access_sync")
public func ckContainerRequestShareAccessSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ shareURL: UnsafePointer<CChar>?,
    _ timeoutSeconds: Double,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard #available(macOS 26.0, *) else {
            throw ckBridgeNSError(code: CKR_FAILURE, message: "Share access requests require macOS 26.0 or newer")
        }
        let container = try ckMakeContainer(containerIdentifier)
        guard let shareURL else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Missing share URL")
        }
        guard let url = URL(string: String(cString: shareURL)) else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Invalid share URL")
        }
        let results = CKItemResults<URL, Void>()
        let operation = CKShareRequestAccessOperation(shareURLs: [url])
        operation.perShareAccessRequestResultBlock = { shareURL, accessRequestResult in
            results.append(shareURL, accessRequestResult)
        }
        operation.shareAccessRequestResultBlock = { operationResult in
            results.finish(operationResult)
        }
        container.add(operation)
        let timedOut = ckWaitCancellingOnTimeout(operation, results.finished, timeoutSeconds: timeoutSeconds)
        let outcome = CKOperationOutcome(results: results.results, operationError: results.operationError, timedOut: timedOut)
        try outcome.single(label: "CKShareRequestAccessOperation", timeoutSeconds: timeoutSeconds, writes: true)
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}
