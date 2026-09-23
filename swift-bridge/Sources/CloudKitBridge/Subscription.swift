import CloudKit
import CloudKitObjCBridge
import Foundation

enum CKSubscriptionPayloadKind: String, Codable {
    case query
    case recordZone
    case database
}

struct CKSubscriptionPayload: Codable {
    var kind: CKSubscriptionPayloadKind
    var subscriptionID: String
    var subscriptionType: Int32
    var notificationInfo: CKNotificationInfoPayload?
    var recordType: String?
    var predicate: CKPredicatePayload?
    var predicateFormat: String?
    var zoneID: CKRecordZoneIDPayload?
    var querySubscriptionOptions: UInt64?
    var archivedData: [UInt8]?
}

func ckEncodeSubscription(_ subscription: CKSubscription) throws -> CKSubscriptionPayload {
    let subscriptionType = Int32(clamping: subscription.subscriptionType.rawValue)
    if let querySubscription = subscription as? CKQuerySubscription {
        return CKSubscriptionPayload(
            kind: .query,
            subscriptionID: querySubscription.subscriptionID,
            subscriptionType: subscriptionType,
            notificationInfo: ckEncodeNotificationInfo(querySubscription.notificationInfo),
            recordType: querySubscription.recordType,
            predicate: nil,
            predicateFormat: querySubscription.predicate.predicateFormat,
            zoneID: querySubscription.zoneID.map(ckEncodeZoneID),
            querySubscriptionOptions: UInt64(querySubscription.querySubscriptionOptions.rawValue),
            archivedData: try ckArchiveSecureCoding(querySubscription)
        )
    }
    if let recordZoneSubscription = subscription as? CKRecordZoneSubscription {
        return CKSubscriptionPayload(
            kind: .recordZone,
            subscriptionID: recordZoneSubscription.subscriptionID,
            subscriptionType: subscriptionType,
            notificationInfo: ckEncodeNotificationInfo(recordZoneSubscription.notificationInfo),
            recordType: recordZoneSubscription.recordType,
            predicate: nil,
            predicateFormat: nil,
            zoneID: ckEncodeZoneID(recordZoneSubscription.zoneID),
            querySubscriptionOptions: nil,
            archivedData: nil
        )
    }
    if let databaseSubscription = subscription as? CKDatabaseSubscription {
        return CKSubscriptionPayload(
            kind: .database,
            subscriptionID: databaseSubscription.subscriptionID,
            subscriptionType: subscriptionType,
            notificationInfo: ckEncodeNotificationInfo(databaseSubscription.notificationInfo),
            recordType: databaseSubscription.recordType,
            predicate: nil,
            predicateFormat: nil,
            zoneID: nil,
            querySubscriptionOptions: nil,
            archivedData: nil
        )
    }
    throw ckBridgeNSError(
        code: CKR_FAILURE,
        message: "CloudKit returned a subscription of unsupported class \(String(describing: type(of: subscription)))"
    )
}

private func ckSetSubscriptionRecordType(_ subscription: CKSubscription, _ recordType: String?) throws {
    var error: NSError?
    guard CKXSubscriptionSetRecordType(subscription, recordType, &error) else {
        throw error ?? ckInvalidArgument("CKSubscription.recordType rejected \(recordType ?? "nil")")
    }
}

private func ckDecodeArchivedQuerySubscription(
    _ archivedData: [UInt8],
    _ payload: CKSubscriptionPayload
) throws -> CKSubscription {
    let decoded = try ckDecodeSecureCodingObject(archivedData, as: CKSubscription.self)
    guard let subscription = decoded as? CKQuerySubscription else {
        throw ckInvalidArgument("Archived subscription is a \(String(describing: type(of: decoded))), not a CKQuerySubscription")
    }
    guard subscription.subscriptionID == payload.subscriptionID else {
        throw ckInvalidArgument("Archived subscription ID \(subscription.subscriptionID) does not match \(payload.subscriptionID)")
    }
    subscription.zoneID = try payload.zoneID.map(ckDecodeZoneID)
    subscription.notificationInfo = ckDecodeNotificationInfo(payload.notificationInfo)
    return subscription
}

func ckDecodeSubscription(_ payload: CKSubscriptionPayload) throws -> CKSubscription {
    var error: NSError?
    switch payload.kind {
    case .query:
        if let archivedData = payload.archivedData, !archivedData.isEmpty {
            return try ckDecodeArchivedQuerySubscription(archivedData, payload)
        }
        guard let predicatePayload = payload.predicate else {
            throw ckInvalidArgument("CKQuerySubscription \(payload.subscriptionID) has no predicate")
        }
        let predicate = try ckDecodePredicate(predicatePayload)
        let options = CKQuerySubscription.Options(rawValue: UInt(payload.querySubscriptionOptions ?? 0))
        let created = CKXQuerySubscriptionCreate(payload.recordType ?? "", predicate, payload.subscriptionID, options, &error)
        let subscription = try ckObjCResult(created, error, "CKQuerySubscription(recordType:predicate:subscriptionID:options:)")
        subscription.zoneID = try payload.zoneID.map(ckDecodeZoneID)
        subscription.notificationInfo = ckDecodeNotificationInfo(payload.notificationInfo)
        return subscription
    case .recordZone:
        guard let zoneIDPayload = payload.zoneID else {
            throw ckInvalidArgument("CKRecordZoneSubscription \(payload.subscriptionID) has no zone ID")
        }
        let created = CKXRecordZoneSubscriptionCreate(try ckDecodeZoneID(zoneIDPayload), payload.subscriptionID, &error)
        let subscription = try ckObjCResult(created, error, "CKRecordZoneSubscription(zoneID:subscriptionID:)")
        try ckSetSubscriptionRecordType(subscription, payload.recordType)
        subscription.notificationInfo = ckDecodeNotificationInfo(payload.notificationInfo)
        return subscription
    case .database:
        let created = CKXDatabaseSubscriptionCreate(payload.subscriptionID, &error)
        let subscription = try ckObjCResult(created, error, "CKDatabaseSubscription(subscriptionID:)")
        try ckSetSubscriptionRecordType(subscription, payload.recordType)
        subscription.notificationInfo = ckDecodeNotificationInfo(payload.notificationInfo)
        return subscription
    }
}
