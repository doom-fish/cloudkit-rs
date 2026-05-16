import CloudKit

extension CKRecordZone.ID {
    func bridgePayload() -> CKRecordZoneIDPayload {
        ckEncodeZoneID(self)
    }

    static func fromBridgePayload(_ payload: CKRecordZoneIDPayload) -> CKRecordZone.ID {
        ckDecodeZoneID(payload)
    }
}

extension CKRecord.ID {
    func bridgePayload() -> CKRecordIDPayload {
        ckEncodeRecordID(self)
    }

    static func fromBridgePayload(_ payload: CKRecordIDPayload) -> CKRecord.ID {
        ckDecodeRecordID(payload)
    }
}
