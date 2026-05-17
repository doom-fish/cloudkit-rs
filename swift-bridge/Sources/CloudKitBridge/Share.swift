import CloudKit
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
        publicPermission: Int32(share.publicPermission.rawValue),
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
    let ownerID = share.owner.participantID
    let currentUserID = share.currentUserParticipant?.participantID
    let desiredIDs = Set(payload.participants.map(\.participantID))

    for participant in share.participants where participant.participantID != ownerID && participant.participantID != currentUserID {
        if !desiredIDs.contains(participant.participantID) {
            share.removeParticipant(participant)
        }
    }

    let existingIDs = Set(share.participants.map(\.participantID))
    for participantPayload in payload.participants where !existingIDs.contains(participantPayload.participantID) {
        share.addParticipant(try ckDecodeShareParticipant(participantPayload))
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
        share = CKShare(recordZoneID: ckDecodeZoneID(payload.zoneID))
    } else if let rootRecord = payload.rootRecord {
        share = CKShare(rootRecord: try ckDecodeRecord(rootRecord))
    } else {
        throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Share payload missing root record for non-zone-wide share")
    }

    share.publicPermission = CKShare.ParticipantPermission(rawValue: Int(payload.publicPermission)) ?? .none
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
    if #available(macOS 26.0, *), let allowsAccessRequests = payload.allowsAccessRequests {
        share.allowsAccessRequests = allowsAccessRequests
    }
    try ckSyncShareParticipants(from: payload, into: share)
    return share
}

func ckEncodeShareMetadata(_ metadata: CKShare.Metadata) throws -> CKShareMetadataPayload {
    CKShareMetadataPayload(
        archivedData: try ckArchiveSecureCoding(metadata),
        containerIdentifier: metadata.containerIdentifier,
        share: try ckEncodeShare(
            metadata.share,
            rootRecord: metadata.rootRecord.flatMap { try? ckEncodeRecord($0) },
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
        rootRecord: metadata.rootRecord.flatMap { try? ckEncodeRecord($0) },
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
        return Int32(error.code)
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
        let share = CKShare(recordZoneID: ckDecodeZoneID(zoneIDPayload))
        outJSON?.pointee = ckCString(try ckEncodeJSON(try ckEncodeShare(share, rootRecord: nil, isZoneWide: true)))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
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
        return Int32(error.code)
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
        return Int32(error.code)
    }
}

@_cdecl("ck_container_fetch_share_metadata_sync")
public func ckContainerFetchShareMetadataSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ shareURL: UnsafePointer<CChar>?,
    _ shouldFetchRootRecord: Bool,
    _ rootRecordDesiredKeysJSON: UnsafePointer<CChar>?,
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

        let holder = CKResultHolder<CKShareMetadataPayload>()
        let semaphore = DispatchSemaphore(value: 0)
        let operation = CKFetchShareMetadataOperation(shareURLs: [url])
        operation.shouldFetchRootRecord = shouldFetchRootRecord
        operation.rootRecordDesiredKeys = desiredKeys
        operation.perShareMetadataResultBlock = { _, shareMetadataResult in
            switch shareMetadataResult {
            case .success(let shareMetadata):
                do {
                    holder.value = try ckEncodeShareMetadata(shareMetadata)
                } catch let error as NSError {
                    holder.error = error
                } catch {
                    holder.error = ckBridgeNSError(code: CKR_FAILURE, message: error.localizedDescription)
                }
            case .failure(let error):
                holder.error = error as NSError
            }
        }
        operation.fetchShareMetadataResultBlock = { operationResult in
            if case .failure(let error) = operationResult, holder.error == nil {
                holder.error = error as NSError
            }
            semaphore.signal()
        }

        container.add(operation)
        if semaphore.wait(timeout: .now() + 30) == .timedOut {
            throw ckTimeoutNSError("CloudKit CKFetchShareMetadataOperation")
        }
        if let error = holder.error {
            throw error
        }
        guard let payload = holder.value else {
            throw ckBridgeNSError(code: CKR_FAILURE, message: "Missing CKShareMetadata result")
        }
        outJSON?.pointee = ckCString(try ckEncodeJSON(payload))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_container_accept_share_metadata_sync")
public func ckContainerAcceptShareMetadataSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ shareMetadataJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let container = try ckMakeContainer(containerIdentifier)
        let payload = try ckDecodeJSON(shareMetadataJSON, as: CKShareMetadataPayload.self)
        let shareMetadata = try ckDecodeShareMetadata(payload)
        let acceptedShare: CKShare = try ckAwait(label: "CloudKit share acceptance") { (completion: @escaping (CKShare?, NSError?) -> Void) in
            container.accept([shareMetadata]) { operationResult in
                switch operationResult {
                case .success(let resultsByMetadata):
                    guard let perShareResult = resultsByMetadata[shareMetadata] ?? resultsByMetadata.values.first else {
                        completion(nil, ckBridgeNSError(code: CKR_FAILURE, message: "Missing per-share acceptance result"))
                        return
                    }
                    switch perShareResult {
                    case .success(let share):
                        completion(share, nil)
                    case .failure(let error):
                        completion(nil, error as NSError)
                    }
                case .failure(let error):
                    completion(nil, error as NSError)
                }
            }
        }
        let acceptedPayload = try ckEncodeShare(
            acceptedShare,
            rootRecord: payload.rootRecord,
            isZoneWide: acceptedShare.recordID.recordName == CKRecordNameZoneWideShare
        )
        outJSON?.pointee = ckCString(try ckEncodeJSON(acceptedPayload))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}

@_cdecl("ck_container_request_share_access_sync")
public func ckContainerRequestShareAccessSync(
    _ containerIdentifier: UnsafePointer<CChar>?,
    _ shareURL: UnsafePointer<CChar>?,
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
        let semaphore = DispatchSemaphore(value: 0)
        let holder = CKResultHolder<Bool>()
        let operation = CKShareRequestAccessOperation(shareURLs: [url])
        operation.perShareAccessRequestResultBlock = { _, accessRequestResult in
            if case .failure(let error) = accessRequestResult {
                holder.error = error as NSError
            }
        }
        operation.shareAccessRequestResultBlock = { operationResult in
            if case .failure(let error) = operationResult, holder.error == nil {
                holder.error = error as NSError
            }
            holder.value = true
            semaphore.signal()
        }
        container.add(operation)
        if semaphore.wait(timeout: .now() + 30) == .timedOut {
            throw ckTimeoutNSError("CloudKit CKShareRequestAccessOperation")
        }
        if let error = holder.error {
            throw error
        }
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return Int32(error.code)
    }
}
