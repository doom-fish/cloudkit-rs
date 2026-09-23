import CloudKit
import CloudKitObjCBridge
import CoreLocation
import Foundation

enum CKPredicateKind: String, Codable {
    case truePredicate
    case falsePredicate
    case comparison
    case beginsWith
    case isIn
    case notIn
    case listContains
    case between
    case containsTokens
    case withinDistance
    case and
    case not
    case format
}

enum CKComparisonOperatorPayload: String, Codable {
    case equal
    case notEqual
    case lessThan
    case lessThanOrEqual
    case greaterThan
    case greaterThanOrEqual

    var symbol: String {
        switch self {
        case .equal: "=="
        case .notEqual: "!="
        case .lessThan: "<"
        case .lessThanOrEqual: "<="
        case .greaterThan: ">"
        case .greaterThanOrEqual: ">="
        }
    }
}

struct CKPredicatePayload: Codable {
    var kind: CKPredicateKind
    var key: String?
    var comparison: CKComparisonOperatorPayload?
    var value: CKRecordValuePayload?
    var values: [CKRecordValuePayload]?
    var text: String?
    var location: CKLocationPayload?
    var radiusKilometers: Double?
    var subpredicates: [CKPredicatePayload]?
    var format: String?
    var arguments: [CKRecordValuePayload]?
}

struct SortDescriptorPayload: Codable {
    var key: String
    var ascending: Bool
    var relativeLocation: CKLocationPayload?
}

struct CKQueryPayload: Codable {
    var recordType: String
    var predicate: CKPredicatePayload
    var sortDescriptors: [SortDescriptorPayload]
}

private func ckPredicateField<T>(_ value: T?, _ field: String, _ kind: CKPredicateKind) throws -> T {
    guard let value else {
        throw ckInvalidArgument("\(kind.rawValue) predicate is missing its \(field)")
    }
    return value
}

private func ckPredicateKey(_ payload: CKPredicatePayload) throws -> NSString {
    let key = try ckPredicateField(payload.key, "key", payload.kind)
    guard !key.isEmpty else {
        throw ckInvalidArgument("\(payload.kind.rawValue) predicate key must not be empty")
    }
    return key as NSString
}

private func ckPredicateArgument(_ payload: CKRecordValuePayload) throws -> Any {
    if payload.kind == .asset {
        throw ckInvalidArgument("CKAsset values cannot be used in query predicates")
    }
    if payload.kind == .array {
        let items = try ckPredicateField(payload.arrayValue, "array value", .comparison)
        return try items.map(ckPredicateArgument) as NSArray
    }
    return try ckDecodeRecordValue(payload, key: "predicate argument")
}

private func ckMakePredicate(_ format: String, _ arguments: [Any]) throws -> NSPredicate {
    var error: NSError?
    let predicate = CKXPredicateWithFormat(format, arguments, &error)
    return try ckObjCResult(predicate, error, "NSPredicate(format:argumentArray:)")
}

func ckDecodePredicate(_ payload: CKPredicatePayload) throws -> NSPredicate {
    switch payload.kind {
    case .truePredicate:
        return try ckMakePredicate("TRUEPREDICATE", [])
    case .falsePredicate:
        return try ckMakePredicate("FALSEPREDICATE", [])
    case .comparison:
        let key = try ckPredicateKey(payload)
        let comparison = try ckPredicateField(payload.comparison, "comparison operator", payload.kind)
        let value = try ckPredicateArgument(try ckPredicateField(payload.value, "value", payload.kind))
        return try ckMakePredicate("%K \(comparison.symbol) %@", [key, value])
    case .beginsWith:
        let key = try ckPredicateKey(payload)
        let prefix = try ckPredicateField(payload.text, "prefix", payload.kind)
        return try ckMakePredicate("%K BEGINSWITH %@", [key, prefix as NSString])
    case .isIn, .notIn:
        let key = try ckPredicateKey(payload)
        let values = try ckPredicateField(payload.values, "values", payload.kind).map(ckPredicateArgument)
        let format = payload.kind == .isIn ? "%K IN %@" : "NOT (%K IN %@)"
        return try ckMakePredicate(format, [key, values as NSArray])
    case .listContains:
        let key = try ckPredicateKey(payload)
        let value = try ckPredicateArgument(try ckPredicateField(payload.value, "value", payload.kind))
        return try ckMakePredicate("%K CONTAINS %@", [key, value])
    case .between:
        let key = try ckPredicateKey(payload)
        let bounds = try ckPredicateField(payload.values, "bounds", payload.kind)
        guard bounds.count == 2 else {
            throw ckInvalidArgument("between predicate needs exactly two bounds, got \(bounds.count)")
        }
        return try ckMakePredicate("%K BETWEEN %@", [key, try bounds.map(ckPredicateArgument) as NSArray])
    case .containsTokens:
        let text = try ckPredicateField(payload.text, "tokens", payload.kind)
        return try ckMakePredicate("self CONTAINS %@", [text as NSString])
    case .withinDistance:
        let key = try ckPredicateKey(payload)
        let location = try ckDecodeLocation(try ckPredicateField(payload.location, "location", payload.kind))
        let radius = try ckPredicateField(payload.radiusKilometers, "radius", payload.kind)
        guard radius.isFinite, radius >= 0 else {
            throw ckInvalidArgument("withinDistance radius must be a finite, non-negative number of kilometers")
        }
        return try ckMakePredicate(
            "distanceToLocation:fromLocation:(%K, %@) < %@",
            [key, location, NSNumber(value: radius)]
        )
    case .and:
        let subpredicates = try ckPredicateField(payload.subpredicates, "subpredicates", payload.kind)
        guard !subpredicates.isEmpty else {
            throw ckInvalidArgument("and predicate needs at least one subpredicate")
        }
        if subpredicates.count == 1 {
            return try ckDecodePredicate(subpredicates[0])
        }
        return NSCompoundPredicate(andPredicateWithSubpredicates: try subpredicates.map(ckDecodePredicate))
    case .not:
        let subpredicates = try ckPredicateField(payload.subpredicates, "subpredicate", payload.kind)
        guard subpredicates.count == 1 else {
            throw ckInvalidArgument("not predicate needs exactly one subpredicate, got \(subpredicates.count)")
        }
        return NSCompoundPredicate(notPredicateWithSubpredicate: try ckDecodePredicate(subpredicates[0]))
    case .format:
        let format = try ckPredicateField(payload.format, "format", payload.kind)
        let arguments = try (payload.arguments ?? []).map(ckPredicateArgument)
        return try ckMakePredicate(format, arguments)
    }
}

func ckDecodeSortDescriptor(_ payload: SortDescriptorPayload) throws -> NSSortDescriptor {
    if let relativeLocation = payload.relativeLocation {
        return CKLocationSortDescriptor(key: payload.key, relativeLocation: try ckDecodeLocation(relativeLocation))
    }
    return NSSortDescriptor(key: payload.key, ascending: payload.ascending)
}

func ckDecodeQuery(_ payload: CKQueryPayload) throws -> CKQuery {
    let predicate = try ckDecodePredicate(payload.predicate)
    var error: NSError?
    let query = try ckObjCResult(
        CKXQueryCreate(payload.recordType, predicate, &error),
        error,
        "CKQuery(recordType:predicate:)"
    )
    if !payload.sortDescriptors.isEmpty {
        query.sortDescriptors = try payload.sortDescriptors.map(ckDecodeSortDescriptor)
    }
    return query
}

@_cdecl("ck_query_predicate_format")
public func ckQueryPredicateFormat(
    _ queryJSON: UnsafePointer<CChar>?,
    _ outJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?,
    _ outErrorJSON: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        let query = try ckDecodeQuery(try ckDecodeJSON(queryJSON, as: CKQueryPayload.self))
        outJSON?.pointee = ckCString(try ckEncodeJSON(query.predicate.predicateFormat))
        return CKR_OK
    } catch let error as NSError {
        ckWriteError(error, to: outErrorJSON)
        return ckFailureStatus(error)
    }
}
