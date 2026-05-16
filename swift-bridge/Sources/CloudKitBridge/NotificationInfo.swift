import CloudKit
import Foundation

struct CKNotificationInfoPayload: Codable {
    var alertBody: String?
    var alertLocalizationKey: String?
    var alertLocalizationArgs: [String]?
    var title: String?
    var titleLocalizationKey: String?
    var titleLocalizationArgs: [String]?
    var subtitle: String?
    var subtitleLocalizationKey: String?
    var subtitleLocalizationArgs: [String]?
    var alertActionLocalizationKey: String?
    var alertLaunchImage: String?
    var soundName: String?
    var desiredKeys: [String]?
    var shouldBadge: Bool
    var shouldSendContentAvailable: Bool
    var shouldSendMutableContent: Bool
    var category: String?
    var collapseIDKey: String?
}

func ckEncodeNotificationInfo(_ info: CKSubscription.NotificationInfo?) -> CKNotificationInfoPayload? {
    guard let info else { return nil }
    return CKNotificationInfoPayload(
        alertBody: info.alertBody,
        alertLocalizationKey: info.alertLocalizationKey,
        alertLocalizationArgs: info.alertLocalizationArgs,
        title: info.title,
        titleLocalizationKey: info.titleLocalizationKey,
        titleLocalizationArgs: info.titleLocalizationArgs,
        subtitle: info.subtitle,
        subtitleLocalizationKey: info.subtitleLocalizationKey,
        subtitleLocalizationArgs: info.subtitleLocalizationArgs,
        alertActionLocalizationKey: info.alertActionLocalizationKey,
        alertLaunchImage: info.alertLaunchImage,
        soundName: info.soundName,
        desiredKeys: info.desiredKeys,
        shouldBadge: info.shouldBadge,
        shouldSendContentAvailable: info.shouldSendContentAvailable,
        shouldSendMutableContent: info.shouldSendMutableContent,
        category: info.category,
        collapseIDKey: info.collapseIDKey
    )
}

func ckDecodeNotificationInfo(_ payload: CKNotificationInfoPayload?) -> CKSubscription.NotificationInfo? {
    guard let payload else { return nil }
    let info = CKSubscription.NotificationInfo()
    info.alertBody = payload.alertBody
    info.alertLocalizationKey = payload.alertLocalizationKey
    info.alertLocalizationArgs = payload.alertLocalizationArgs
    info.title = payload.title
    info.titleLocalizationKey = payload.titleLocalizationKey
    info.titleLocalizationArgs = payload.titleLocalizationArgs
    info.subtitle = payload.subtitle
    info.subtitleLocalizationKey = payload.subtitleLocalizationKey
    info.subtitleLocalizationArgs = payload.subtitleLocalizationArgs
    info.alertActionLocalizationKey = payload.alertActionLocalizationKey
    info.alertLaunchImage = payload.alertLaunchImage
    info.soundName = payload.soundName
    info.desiredKeys = payload.desiredKeys
    info.shouldBadge = payload.shouldBadge
    info.shouldSendContentAvailable = payload.shouldSendContentAvailable
    info.shouldSendMutableContent = payload.shouldSendMutableContent
    info.category = payload.category
    info.collapseIDKey = payload.collapseIDKey
    return info
}
