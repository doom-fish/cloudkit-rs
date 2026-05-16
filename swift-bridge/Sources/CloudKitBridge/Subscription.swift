import CloudKit
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
    var predicateFormat: String?
    var zoneID: CKRecordZoneIDPayload?
    var querySubscriptionOptions: UInt64?
}

func ckEncodeSubscription(_ subscription: CKSubscription) -> CKSubscriptionPayload {
    if let querySubscription = subscription as? CKQuerySubscription {
        return CKSubscriptionPayload(
            kind: .query,
            subscriptionID: querySubscription.subscriptionID,
            subscriptionType: Int32(querySubscription.subscriptionType.rawValue),
            notificationInfo: ckEncodeNotificationInfo(querySubscription.notificationInfo),
            recordType: querySubscription.recordType,
            predicateFormat: querySubscription.predicate.predicateFormat,
            zoneID: querySubscription.zoneID.map(ckEncodeZoneID),
            querySubscriptionOptions: UInt64(querySubscription.querySubscriptionOptions.rawValue)
        )
    }
    if let recordZoneSubscription = subscription as? CKRecordZoneSubscription {
        return CKSubscriptionPayload(
            kind: .recordZone,
            subscriptionID: recordZoneSubscription.subscriptionID,
            subscriptionType: Int32(recordZoneSubscription.subscriptionType.rawValue),
            notificationInfo: ckEncodeNotificationInfo(recordZoneSubscription.notificationInfo),
            recordType: recordZoneSubscription.recordType,
            predicateFormat: nil,
            zoneID: ckEncodeZoneID(recordZoneSubscription.zoneID),
            querySubscriptionOptions: nil
        )
    }
    let databaseSubscription = subscription as! CKDatabaseSubscription
    return CKSubscriptionPayload(
        kind: .database,
        subscriptionID: databaseSubscription.subscriptionID,
        subscriptionType: Int32(databaseSubscription.subscriptionType.rawValue),
        notificationInfo: ckEncodeNotificationInfo(databaseSubscription.notificationInfo),
        recordType: databaseSubscription.recordType,
        predicateFormat: nil,
        zoneID: nil,
        querySubscriptionOptions: nil
    )
}

func ckDecodeSubscription(_ payload: CKSubscriptionPayload) -> CKSubscription {
    switch payload.kind {
    case .query:
        let subscription = CKQuerySubscription(
            recordType: payload.recordType ?? "",
            predicate: NSPredicate(format: payload.predicateFormat ?? "TRUEPREDICATE"),
            subscriptionID: payload.subscriptionID,
            options: CKQuerySubscription.Options(rawValue: UInt(payload.querySubscriptionOptions ?? 0))
        )
        subscription.zoneID = payload.zoneID.map(ckDecodeZoneID)
        subscription.notificationInfo = ckDecodeNotificationInfo(payload.notificationInfo)
        return subscription
    case .recordZone:
        let subscription = CKRecordZoneSubscription(
            zoneID: payload.zoneID.map(ckDecodeZoneID) ?? CKRecordZone.ID(zoneName: CKRecordZone.ID.defaultZoneName, ownerName: CKCurrentUserDefaultName),
            subscriptionID: payload.subscriptionID
        )
        subscription.recordType = payload.recordType
        subscription.notificationInfo = ckDecodeNotificationInfo(payload.notificationInfo)
        return subscription
    case .database:
        let subscription = CKDatabaseSubscription(subscriptionID: payload.subscriptionID)
        subscription.recordType = payload.recordType
        subscription.notificationInfo = ckDecodeNotificationInfo(payload.notificationInfo)
        return subscription
    }
}
