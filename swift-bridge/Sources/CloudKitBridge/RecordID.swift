import CloudKit

extension CKRecordZone.ID {
    func bridgePayload() -> CKRecordZoneIDPayload {
        ckEncodeZoneID(self)
    }

    static func fromBridgePayload(_ payload: CKRecordZoneIDPayload) throws -> CKRecordZone.ID {
        try ckDecodeZoneID(payload)
    }
}

extension CKRecord.ID {
    func bridgePayload() -> CKRecordIDPayload {
        ckEncodeRecordID(self)
    }

    static func fromBridgePayload(_ payload: CKRecordIDPayload) throws -> CKRecord.ID {
        try ckDecodeRecordID(payload)
    }
}
