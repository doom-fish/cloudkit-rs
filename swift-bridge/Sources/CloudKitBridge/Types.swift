import CloudKit
import CloudKitObjCBridge
import CoreLocation
import Foundation

let ckISO8601Formatter: ISO8601DateFormatter = {
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return formatter
}()

enum CKRecordValueKind: String, Codable {
    case string
    case int
    case double
    case bool
    case bytes
    case date
    case asset
    case reference
    case location
    case array
}

struct CKRecordZoneIDPayload: Codable {
    var zoneName: String
    var ownerName: String
}

struct CKRecordIDPayload: Codable {
    var recordName: String
    var zoneID: CKRecordZoneIDPayload
}

struct CKReferencePayload: Codable {
    var recordID: CKRecordIDPayload
    var action: UInt
}

struct CKAssetPayload: Codable {
    var fileURL: String?
}

struct CKLocationPayload: Codable {
    var latitude: Double
    var longitude: Double
    var altitude: Double
    var horizontalAccuracy: Double
    var verticalAccuracy: Double
    var course: Double
    var speed: Double
    var timestamp: String?
}

struct CKRecordValuePayload: Codable {
    var kind: CKRecordValueKind
    var stringValue: String?
    var intValue: Int64?
    var doubleValue: Double?
    var boolValue: Bool?
    var bytesValue: [UInt8]?
    var dateValue: String?
    var assetValue: CKAssetPayload?
    var referenceValue: CKReferencePayload?
    var locationValue: CKLocationPayload?
    var arrayValue: [CKRecordValuePayload]?

    init(kind: CKRecordValueKind) {
        self.kind = kind
    }
}

struct CKRecordPayload: Codable {
    var recordType: String
    var recordID: CKRecordIDPayload
    var fields: [String: CKRecordValuePayload]
    var encodedSystemFields: [UInt8]
    var recordChangeTag: String?
    var creatorUserRecordID: CKRecordIDPayload?
    var creationDate: String?
    var lastModifiedUserRecordID: CKRecordIDPayload?
    var modificationDate: String?
    var parent: CKReferencePayload?
    var share: CKReferencePayload?
    var changedKeys: [String]
    var allTokens: [String]
}

struct CKRecordZonePayload: Codable {
    var zoneID: CKRecordZoneIDPayload
    var capabilities: UInt64
    var share: CKReferencePayload?
    var encryptionScope: Int?
}

struct CKModifyRecordsOperationPayload: Codable {
    var recordsToSave: [CKRecordPayload]
    var recordIDsToDelete: [CKRecordIDPayload]
    var savePolicy: Int32
    var atomic: Bool
}

struct CKRecordSaveResultPayload: Codable {
    var recordID: CKRecordIDPayload
    var record: CKRecordPayload?
    var error: CKErrorPayload?
}

struct CKRecordDeleteResultPayload: Codable {
    var recordID: CKRecordIDPayload
    var error: CKErrorPayload?
}

struct CKModifyRecordsResultPayload: Codable {
    var savedRecords: [CKRecordPayload]
    var deletedRecordIDs: [CKRecordIDPayload]
    var saveResults: [CKRecordSaveResultPayload]
    var deleteResults: [CKRecordDeleteResultPayload]
    var operationError: CKErrorPayload?
}

struct CKQueryOperationPayload: Codable {
    var query: CKQueryPayload
    var cursor: CKQueryCursorPayload?
    var zoneID: CKRecordZoneIDPayload?
    var desiredKeys: [String]?
    var resultsLimit: Int?
}

struct CKQueryMatchResultPayload: Codable {
    var recordID: CKRecordIDPayload
    var record: CKRecordPayload?
    var error: CKErrorPayload?
}

struct CKQueryOperationResultPayload: Codable {
    var records: [CKRecordPayload]
    var matches: [CKQueryMatchResultPayload]
    var cursorReturned: Bool
    var cursor: CKQueryCursorPayload?
    var operationError: CKErrorPayload?
}

func ckEncodeZoneID(_ zoneID: CKRecordZone.ID) -> CKRecordZoneIDPayload {
    CKRecordZoneIDPayload(zoneName: zoneID.zoneName, ownerName: zoneID.ownerName)
}

func ckDecodeZoneID(_ payload: CKRecordZoneIDPayload) throws -> CKRecordZone.ID {
    var error: NSError?
    let zoneID = CKXRecordZoneIDCreate(payload.zoneName, payload.ownerName, &error)
    return try ckObjCResult(zoneID, error, "CKRecordZone.ID(zoneName:ownerName:)")
}

func ckEncodeRecordID(_ recordID: CKRecord.ID) -> CKRecordIDPayload {
    CKRecordIDPayload(recordName: recordID.recordName, zoneID: ckEncodeZoneID(recordID.zoneID))
}

func ckDecodeRecordID(_ payload: CKRecordIDPayload) throws -> CKRecord.ID {
    let zoneID = try ckDecodeZoneID(payload.zoneID)
    var error: NSError?
    let recordID = CKXRecordIDCreate(payload.recordName, zoneID, &error)
    return try ckObjCResult(recordID, error, "CKRecord.ID(recordName:zoneID:)")
}

func ckEncodeReference(_ reference: CKRecord.Reference) -> CKReferencePayload {
    CKReferencePayload(recordID: ckEncodeRecordID(reference.recordID), action: reference.action.rawValue)
}

func ckDecodeReference(_ payload: CKReferencePayload) throws -> CKRecord.Reference {
    let action: CKRecord.ReferenceAction
    switch payload.action {
    case CKRecord.ReferenceAction.none.rawValue:
        action = .none
    case CKRecord.ReferenceAction.deleteSelf.rawValue:
        action = .deleteSelf
    default:
        throw ckInvalidArgument("Unsupported CKReferenceAction raw value \(payload.action)")
    }
    return CKRecord.Reference(recordID: try ckDecodeRecordID(payload.recordID), action: action)
}

func ckEncodeLocation(_ location: CLLocation) -> CKLocationPayload {
    CKLocationPayload(
        latitude: location.coordinate.latitude,
        longitude: location.coordinate.longitude,
        altitude: location.altitude,
        horizontalAccuracy: location.horizontalAccuracy,
        verticalAccuracy: location.verticalAccuracy,
        course: location.course,
        speed: location.speed,
        timestamp: ckISO8601Formatter.string(from: location.timestamp)
    )
}

func ckDecodeDate(_ value: String) throws -> Date {
    guard let date = ckISO8601Formatter.date(from: value) else {
        throw ckInvalidArgument("Invalid ISO-8601 date: \(value)")
    }
    return date
}

func ckDecodeLocation(_ payload: CKLocationPayload) throws -> CLLocation {
    let timestamp = try payload.timestamp.map(ckDecodeDate) ?? Date()
    return CLLocation(
        coordinate: CLLocationCoordinate2D(latitude: payload.latitude, longitude: payload.longitude),
        altitude: payload.altitude,
        horizontalAccuracy: payload.horizontalAccuracy,
        verticalAccuracy: payload.verticalAccuracy,
        course: payload.course,
        speed: payload.speed,
        timestamp: timestamp
    )
}

func ckArchiveSystemFields(_ record: CKRecord) throws -> [UInt8] {
    let archiver = NSKeyedArchiver(requiringSecureCoding: true)
    record.encodeSystemFields(with: archiver)
    archiver.finishEncoding()
    return [UInt8](archiver.encodedData)
}

func ckEncodeRecordValue(_ value: Any, key: String) throws -> CKRecordValuePayload {
    if let string = value as? String {
        var payload = CKRecordValuePayload(kind: .string)
        payload.stringValue = string
        return payload
    }
    if let number = value as? NSNumber {
        if CFGetTypeID(number) == CFBooleanGetTypeID() {
            var payload = CKRecordValuePayload(kind: .bool)
            payload.boolValue = number.boolValue
            return payload
        }
        if CFNumberIsFloatType(number) {
            var payload = CKRecordValuePayload(kind: .double)
            payload.doubleValue = number.doubleValue
            return payload
        }
        var payload = CKRecordValuePayload(kind: .int)
        payload.intValue = number.int64Value
        return payload
    }
    if let data = value as? Data {
        var payload = CKRecordValuePayload(kind: .bytes)
        payload.bytesValue = [UInt8](data)
        return payload
    }
    if let date = value as? Date {
        var payload = CKRecordValuePayload(kind: .date)
        payload.dateValue = ckISO8601Formatter.string(from: date)
        return payload
    }
    if let asset = value as? CKAsset {
        var payload = CKRecordValuePayload(kind: .asset)
        payload.assetValue = ckEncodeAsset(asset)
        return payload
    }
    if let reference = value as? CKRecord.Reference {
        var payload = CKRecordValuePayload(kind: .reference)
        payload.referenceValue = ckEncodeReference(reference)
        return payload
    }
    if let location = value as? CLLocation {
        var payload = CKRecordValuePayload(kind: .location)
        payload.locationValue = ckEncodeLocation(location)
        return payload
    }
    if let array = value as? [Any] {
        var payload = CKRecordValuePayload(kind: .array)
        payload.arrayValue = try array.enumerated().map { index, item in
            try ckEncodeRecordValue(item, key: "\(key)[\(index)]")
        }
        return payload
    }
    throw ckBridgeNSError(
        code: CKR_FAILURE,
        message: "CKRecord field \(key) holds a \(String(describing: type(of: value))) value, which the bridge cannot represent"
    )
}

private func ckRequired<T>(_ value: T?, _ kind: CKRecordValueKind, _ key: String) throws -> T {
    guard let value else {
        throw ckInvalidArgument("CKRecord value for \(key) has kind \(kind.rawValue) but no \(kind.rawValue) payload")
    }
    return value
}

func ckDecodeRecordValue(_ payload: CKRecordValuePayload, key: String) throws -> any CKRecordValue {
    switch payload.kind {
    case .string:
        return try ckRequired(payload.stringValue, payload.kind, key) as NSString
    case .int:
        return NSNumber(value: try ckRequired(payload.intValue, payload.kind, key))
    case .double:
        return NSNumber(value: try ckRequired(payload.doubleValue, payload.kind, key))
    case .bool:
        return NSNumber(value: try ckRequired(payload.boolValue, payload.kind, key))
    case .bytes:
        return NSData(data: Data(try ckRequired(payload.bytesValue, payload.kind, key)))
    case .date:
        return try ckDecodeDate(try ckRequired(payload.dateValue, payload.kind, key)) as NSDate
    case .asset:
        return try ckDecodeAsset(try ckRequired(payload.assetValue, payload.kind, key), key: key)
    case .reference:
        return try ckDecodeReference(try ckRequired(payload.referenceValue, payload.kind, key))
    case .location:
        let location = try ckRequired(payload.locationValue, payload.kind, key)
        guard location.timestamp != nil else {
            throw ckInvalidArgument("CKRecord location value for \(key) has no timestamp")
        }
        return try ckDecodeLocation(location)
    case .array:
        let items = try ckRequired(payload.arrayValue, payload.kind, key)
        return try items.enumerated().map { index, item in
            try ckDecodeRecordValue(item, key: "\(key)[\(index)]")
        } as NSArray
    }
}

func ckSetRecordObject(_ record: CKRecord, _ value: (any CKRecordValue)?, forKey key: String) throws {
    var error: NSError?
    guard CKXRecordSetObject(record, value, key, &error) else {
        throw error ?? ckInvalidArgument("CKRecord.setObject(_:forKey:) rejected key \(key)")
    }
}

func ckApplyRecordPayload(_ payload: CKRecordPayload, to record: CKRecord, allFields: Bool) throws {
    let changedKeys = Set(payload.changedKeys)
    let keysToSet = allFields || payload.encodedSystemFields.isEmpty
        ? payload.fields.keys.sorted()
        : changedKeys.filter { payload.fields[$0] != nil }.sorted()
    for key in keysToSet {
        guard let valuePayload = payload.fields[key] else { continue }
        try ckSetRecordObject(record, try ckDecodeRecordValue(valuePayload, key: key), forKey: key)
    }
    for key in changedKeys.sorted() where payload.fields[key] == nil {
        try ckSetRecordObject(record, nil, forKey: key)
    }

    let parent = try payload.parent.map { reference -> CKRecord.Reference in
        guard reference.action == CKRecord.ReferenceAction.none.rawValue else {
            throw ckInvalidArgument("CKRecord.parent must use CKReferenceAction.none; CloudKit raises an exception for any other action")
        }
        return try ckDecodeReference(reference)
    }
    if record.parent?.recordID != parent?.recordID {
        var error: NSError?
        guard CKXRecordSetParent(record, parent, &error) else {
            throw error ?? ckInvalidArgument("CKRecord.parent rejected the reference")
        }
    }
}

func ckDecodeRecord(_ payload: CKRecordPayload, allFields: Bool) throws -> CKRecord {
    let record: CKRecord
    if payload.encodedSystemFields.isEmpty {
        var error: NSError?
        let created = CKXRecordCreate(payload.recordType, try ckDecodeRecordID(payload.recordID), &error)
        record = try ckObjCResult(created, error, "CKRecord(recordType:recordID:)")
    } else {
        let data = Data(payload.encodedSystemFields)
        let unarchiver = try NSKeyedUnarchiver(forReadingFrom: data)
        unarchiver.requiresSecureCoding = true
        if payload.recordType == CKRecord.SystemType.share {
            record = CKShare(coder: unarchiver)
        } else {
            guard let decoded = CKRecord(coder: unarchiver) else {
                throw ckBridgeNSError(code: CKR_FAILURE, message: "Failed to decode CKRecord from encoded system fields")
            }
            record = decoded
        }
        unarchiver.finishDecoding()
    }

    try ckApplyRecordPayload(payload, to: record, allFields: allFields)
    return record
}

func ckEncodeRecord(_ record: CKRecord, keepingUnchangedFieldsOf original: CKRecordPayload) throws -> CKRecordPayload {
    var payload = try ckEncodeRecord(record)
    let changedKeys = Set(payload.changedKeys)
    for (key, value) in original.fields where payload.fields[key] == nil && !changedKeys.contains(key) {
        payload.fields[key] = value
    }
    return payload
}

func ckEncodeSavedRecord(_ record: CKRecord, original: CKRecordPayload?) throws -> CKRecordPayload {
    var payload = try original.map { try ckEncodeRecord(record, keepingUnchangedFieldsOf: $0) } ?? ckEncodeRecord(record)
    payload.changedKeys = []
    return payload
}

func ckEncodeRecord(_ record: CKRecord) throws -> CKRecordPayload {
    var fields: [String: CKRecordValuePayload] = [:]
    for key in record.allKeys() {
        guard let value = record.object(forKey: key) else { continue }
        fields[key] = try ckEncodeRecordValue(value, key: key)
    }
    return CKRecordPayload(
        recordType: record.recordType,
        recordID: ckEncodeRecordID(record.recordID),
        fields: fields,
        encodedSystemFields: try ckArchiveSystemFields(record),
        recordChangeTag: record.recordChangeTag,
        creatorUserRecordID: record.creatorUserRecordID.map(ckEncodeRecordID),
        creationDate: record.creationDate.map(ckISO8601Formatter.string),
        lastModifiedUserRecordID: record.lastModifiedUserRecordID.map(ckEncodeRecordID),
        modificationDate: record.modificationDate.map(ckISO8601Formatter.string),
        parent: record.parent.map(ckEncodeReference),
        share: record.share.map(ckEncodeReference),
        changedKeys: record.changedKeys(),
        allTokens: record.allTokens()
    )
}

func ckEncodeRecordResult(_ record: CKRecord) -> (CKRecordPayload?, CKErrorPayload?) {
    do {
        return (try ckEncodeRecord(record), nil)
    } catch let error as NSError {
        return (nil, ckErrorPayload(from: error))
    }
}

func ckEncodeZone(_ zone: CKRecordZone) -> CKRecordZonePayload {
    let encryptionScope: Int?
    if #available(macOS 26.0, *) {
        encryptionScope = zone.encryptionScope.rawValue
    } else {
        encryptionScope = nil
    }
    return CKRecordZonePayload(
        zoneID: ckEncodeZoneID(zone.zoneID),
        capabilities: UInt64(zone.capabilities.rawValue),
        share: zone.share.map(ckEncodeReference),
        encryptionScope: encryptionScope
    )
}

func ckDecodeZone(_ payload: CKRecordZonePayload) throws -> CKRecordZone {
    let zone = CKRecordZone(zoneID: try ckDecodeZoneID(payload.zoneID))
    if #available(macOS 26.0, *), let encryptionScope = payload.encryptionScope,
       let scope = CKRecordZone.EncryptionScope(rawValue: encryptionScope) {
        zone.encryptionScope = scope
    }
    return zone
}
