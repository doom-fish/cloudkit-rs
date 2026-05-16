import CloudKit
import Foundation

struct CKPersonNameComponentsPayload: Codable {
    var namePrefix: String?
    var givenName: String?
    var middleName: String?
    var familyName: String?
    var nameSuffix: String?
    var nickname: String?
}

struct CKUserIdentityLookupInfoPayload: Codable {
    var emailAddress: String?
    var phoneNumber: String?
    var userRecordID: CKRecordIDPayload?
}

struct CKUserIdentityPayload: Codable {
    var archivedData: [UInt8]
    var userRecordID: CKRecordIDPayload?
    var lookupInfo: CKUserIdentityLookupInfoPayload?
    var nameComponents: CKPersonNameComponentsPayload?
    var hasiCloudAccount: Bool
    var contactIdentifiers: [String]
}

struct CKShareParticipantPayload: Codable {
    var archivedData: [UInt8]
    var userIdentity: CKUserIdentityPayload
    var role: Int?
    var permission: Int
    var acceptanceStatus: Int
    var participantID: String
    var isApprovedRequester: Bool?
    var dateAddedToShare: String?
}

func ckEncodeNameComponents(_ components: PersonNameComponents?) -> CKPersonNameComponentsPayload? {
    guard let components else { return nil }
    return CKPersonNameComponentsPayload(
        namePrefix: components.namePrefix,
        givenName: components.givenName,
        middleName: components.middleName,
        familyName: components.familyName,
        nameSuffix: components.nameSuffix,
        nickname: components.nickname
    )
}

func ckEncodeLookupInfo(_ lookupInfo: CKUserIdentity.LookupInfo?) -> CKUserIdentityLookupInfoPayload? {
    guard let lookupInfo else { return nil }
    return CKUserIdentityLookupInfoPayload(
        emailAddress: lookupInfo.emailAddress,
        phoneNumber: lookupInfo.phoneNumber,
        userRecordID: lookupInfo.userRecordID.map(ckEncodeRecordID)
    )
}

func ckDecodeLookupInfo(_ payload: CKUserIdentityLookupInfoPayload) throws -> CKUserIdentity.LookupInfo {
    if let emailAddress = payload.emailAddress {
        return CKUserIdentity.LookupInfo(emailAddress: emailAddress)
    }
    if let phoneNumber = payload.phoneNumber {
        return CKUserIdentity.LookupInfo(phoneNumber: phoneNumber)
    }
    if let userRecordID = payload.userRecordID {
        return CKUserIdentity.LookupInfo(userRecordID: ckDecodeRecordID(userRecordID))
    }
    throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "User identity lookup info requires an email address, phone number, or user record ID")
}

func ckEncodeUserIdentity(_ identity: CKUserIdentity) -> CKUserIdentityPayload {
    CKUserIdentityPayload(
        archivedData: (try? ckArchiveSecureCoding(identity)) ?? [],
        userRecordID: identity.userRecordID.map(ckEncodeRecordID),
        lookupInfo: ckEncodeLookupInfo(identity.lookupInfo),
        nameComponents: ckEncodeNameComponents(identity.nameComponents),
        hasiCloudAccount: identity.hasiCloudAccount,
        contactIdentifiers: identity.contactIdentifiers
    )
}

func ckEncodeShareParticipant(_ participant: CKShare.Participant) -> CKShareParticipantPayload {
    let isApprovedRequester: Bool?
    let dateAddedToShare: String?
    if #available(macOS 26.0, *) {
        isApprovedRequester = participant.isApprovedRequester
        dateAddedToShare = participant.dateAddedToShare.map { ckISO8601Formatter.string(from: $0) }
    } else {
        isApprovedRequester = nil
        dateAddedToShare = nil
    }
    return CKShareParticipantPayload(
        archivedData: (try? ckArchiveSecureCoding(participant)) ?? [],
        userIdentity: ckEncodeUserIdentity(participant.userIdentity),
        role: participant.role.rawValue,
        permission: participant.permission.rawValue,
        acceptanceStatus: participant.acceptanceStatus.rawValue,
        participantID: participant.participantID,
        isApprovedRequester: isApprovedRequester,
        dateAddedToShare: dateAddedToShare
    )
}

func ckDecodeShareParticipant(_ payload: CKShareParticipantPayload) throws -> CKShare.Participant {
    if !payload.archivedData.isEmpty {
        return try ckDecodeSecureCodingObject(payload.archivedData, as: CKShare.Participant.self)
    }
    if #available(macOS 15.0, *) {
        return CKShare.Participant.oneTimeURLParticipant()
    }
    throw ckBridgeNSError(code: CKR_FAILURE, message: "Share participant payload did not contain archived data")
}
