import CloudKit
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
    var fileURL: String
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
    var arrayValue: [CKRecordValuePayload]?
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

struct SortDescriptorPayload: Codable {
    var key: String
    var ascending: Bool
}

struct CKQueryPayload: Codable {
    var recordType: String
    var predicateFormat: String
    var sortDescriptors: [SortDescriptorPayload]
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
    var operationError: CKErrorPayload?
}

func ckEncodeZoneID(_ zoneID: CKRecordZone.ID) -> CKRecordZoneIDPayload {
    CKRecordZoneIDPayload(zoneName: zoneID.zoneName, ownerName: zoneID.ownerName)
}

func ckDecodeZoneID(_ payload: CKRecordZoneIDPayload) -> CKRecordZone.ID {
    CKRecordZone.ID(zoneName: payload.zoneName, ownerName: payload.ownerName)
}

func ckEncodeRecordID(_ recordID: CKRecord.ID) -> CKRecordIDPayload {
    CKRecordIDPayload(recordName: recordID.recordName, zoneID: ckEncodeZoneID(recordID.zoneID))
}

func ckDecodeRecordID(_ payload: CKRecordIDPayload) -> CKRecord.ID {
    CKRecord.ID(recordName: payload.recordName, zoneID: ckDecodeZoneID(payload.zoneID))
}

func ckEncodeReference(_ reference: CKRecord.Reference) -> CKReferencePayload {
    CKReferencePayload(recordID: ckEncodeRecordID(reference.recordID), action: reference.action.rawValue)
}

func ckDecodeReference(_ payload: CKReferencePayload) -> CKRecord.Reference {
    CKRecord.Reference(recordID: ckDecodeRecordID(payload.recordID), action: CKRecord.ReferenceAction(rawValue: payload.action) ?? .none)
}

func ckArchiveSystemFields(_ record: CKRecord) throws -> [UInt8] {
    let archiver = NSKeyedArchiver(requiringSecureCoding: true)
    record.encodeSystemFields(with: archiver)
    archiver.finishEncoding()
    return [UInt8](archiver.encodedData)
}

func ckEncodeRecordValue(_ value: Any) -> CKRecordValuePayload? {
    if let string = value as? String {
        return CKRecordValuePayload(
            kind: .string,
            stringValue: string,
            intValue: nil,
            doubleValue: nil,
            boolValue: nil,
            bytesValue: nil,
            dateValue: nil,
            assetValue: nil,
            referenceValue: nil,
            arrayValue: nil
        )
    }
    if let number = value as? NSNumber {
        if CFGetTypeID(number) == CFBooleanGetTypeID() {
            return CKRecordValuePayload(
                kind: .bool,
                stringValue: nil,
                intValue: nil,
                doubleValue: nil,
                boolValue: number.boolValue,
                bytesValue: nil,
                dateValue: nil,
                assetValue: nil,
                referenceValue: nil,
                arrayValue: nil
            )
        }
        if CFNumberIsFloatType(number) {
            return CKRecordValuePayload(
                kind: .double,
                stringValue: nil,
                intValue: nil,
                doubleValue: number.doubleValue,
                boolValue: nil,
                bytesValue: nil,
                dateValue: nil,
                assetValue: nil,
                referenceValue: nil,
                arrayValue: nil
            )
        }
        return CKRecordValuePayload(
            kind: .int,
            stringValue: nil,
            intValue: number.int64Value,
            doubleValue: nil,
            boolValue: nil,
            bytesValue: nil,
            dateValue: nil,
            assetValue: nil,
            referenceValue: nil,
            arrayValue: nil
        )
    }
    if let data = value as? Data {
        return CKRecordValuePayload(
            kind: .bytes,
            stringValue: nil,
            intValue: nil,
            doubleValue: nil,
            boolValue: nil,
            bytesValue: [UInt8](data),
            dateValue: nil,
            assetValue: nil,
            referenceValue: nil,
            arrayValue: nil
        )
    }
    if let date = value as? Date {
        return CKRecordValuePayload(
            kind: .date,
            stringValue: nil,
            intValue: nil,
            doubleValue: nil,
            boolValue: nil,
            bytesValue: nil,
            dateValue: ckISO8601Formatter.string(from: date),
            assetValue: nil,
            referenceValue: nil,
            arrayValue: nil
        )
    }
    if let asset = value as? CKAsset {
        return CKRecordValuePayload(
            kind: .asset,
            stringValue: nil,
            intValue: nil,
            doubleValue: nil,
            boolValue: nil,
            bytesValue: nil,
            dateValue: nil,
            assetValue: ckEncodeAsset(asset),
            referenceValue: nil,
            arrayValue: nil
        )
    }
    if let reference = value as? CKRecord.Reference {
        return CKRecordValuePayload(
            kind: .reference,
            stringValue: nil,
            intValue: nil,
            doubleValue: nil,
            boolValue: nil,
            bytesValue: nil,
            dateValue: nil,
            assetValue: nil,
            referenceValue: ckEncodeReference(reference),
            arrayValue: nil
        )
    }
    if let array = value as? [Any] {
        var payloads: [CKRecordValuePayload] = []
        payloads.reserveCapacity(array.count)
        for item in array {
            guard let payload = ckEncodeRecordValue(item) else { return nil }
            payloads.append(payload)
        }
        return CKRecordValuePayload(
            kind: .array,
            stringValue: nil,
            intValue: nil,
            doubleValue: nil,
            boolValue: nil,
            bytesValue: nil,
            dateValue: nil,
            assetValue: nil,
            referenceValue: nil,
            arrayValue: payloads
        )
    }
    return nil
}

func ckDecodeRecordValue(_ payload: CKRecordValuePayload) throws -> (any CKRecordValue)? {
    switch payload.kind {
    case .string:
        return payload.stringValue.map(NSString.init(string:))
    case .int:
        return payload.intValue.map(NSNumber.init(value:))
    case .double:
        return payload.doubleValue.map(NSNumber.init(value:))
    case .bool:
        return payload.boolValue.map(NSNumber.init(value:))
    case .bytes:
        return payload.bytesValue.map { NSData(data: Data($0)) }
    case .date:
        guard let dateValue = payload.dateValue else { return nil }
        guard let date = ckISO8601Formatter.date(from: dateValue) else {
            throw ckBridgeNSError(code: CKR_INVALID_ARGUMENT, message: "Invalid ISO-8601 date: \(dateValue)")
        }
        return date as NSDate
    case .asset:
        guard let asset = payload.assetValue else { return nil }
        return ckDecodeAsset(asset)
    case .reference:
        guard let referenceValue = payload.referenceValue else { return nil }
        return ckDecodeReference(referenceValue)
    case .array:
        let values = try payload.arrayValue?.map { try ckDecodeRecordValue($0) } ?? []
        return values.compactMap { $0 } as NSArray
    }
}

func ckApplyRecordPayload(_ payload: CKRecordPayload, to record: CKRecord) throws {
    let desiredKeys = Set(payload.fields.keys)
    for key in record.allKeys() where !desiredKeys.contains(key) {
        record.setObject(nil, forKey: key)
    }
    for (key, valuePayload) in payload.fields {
        record.setObject(try ckDecodeRecordValue(valuePayload), forKey: key)
    }
    record.parent = payload.parent.map(ckDecodeReference)
}

func ckDecodeRecord(_ payload: CKRecordPayload) throws -> CKRecord {
    let recordID = ckDecodeRecordID(payload.recordID)
    let record: CKRecord
    if payload.encodedSystemFields.isEmpty {
        record = CKRecord(recordType: payload.recordType, recordID: recordID)
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

    try ckApplyRecordPayload(payload, to: record)
    return record
}

func ckEncodeRecord(_ record: CKRecord) throws -> CKRecordPayload {
    var fields: [String: CKRecordValuePayload] = [:]
    for key in record.allKeys() {
        if let value = record.object(forKey: key), let payload = ckEncodeRecordValue(value) {
            fields[key] = payload
        }
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

func ckDecodeZone(_ payload: CKRecordZonePayload) -> CKRecordZone {
    let zone = CKRecordZone(zoneID: ckDecodeZoneID(payload.zoneID))
    if #available(macOS 26.0, *), let encryptionScope = payload.encryptionScope,
       let scope = CKRecordZone.EncryptionScope(rawValue: encryptionScope) {
        zone.encryptionScope = scope
    }
    return zone
}

func ckDecodeQuery(_ payload: CKQueryPayload) -> CKQuery {
    let query = CKQuery(recordType: payload.recordType, predicate: NSPredicate(format: payload.predicateFormat))
    if !payload.sortDescriptors.isEmpty {
        query.sortDescriptors = payload.sortDescriptors.map { NSSortDescriptor(key: $0.key, ascending: $0.ascending) }
    }
    return query
}
