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
