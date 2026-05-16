import CloudKit
import Foundation

struct CKServerChangeTokenPayload: Codable {
    var archivedData: [UInt8]
}

struct CKQueryCursorPayload: Codable {
    var archivedData: [UInt8]
}

func ckEncodeServerChangeToken(_ token: CKServerChangeToken) -> CKServerChangeTokenPayload {
    CKServerChangeTokenPayload(archivedData: (try? ckArchiveSecureCoding(token)) ?? [])
}

func ckDecodeServerChangeToken(_ payload: CKServerChangeTokenPayload) throws -> CKServerChangeToken {
    try ckDecodeSecureCodingObject(payload.archivedData, as: CKServerChangeToken.self)
}

func ckEncodeQueryCursor(_ cursor: CKQueryOperation.Cursor) -> CKQueryCursorPayload {
    CKQueryCursorPayload(archivedData: (try? ckArchiveSecureCoding(cursor)) ?? [])
}
