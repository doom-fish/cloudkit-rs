import CloudKit

extension CKRecordZone {
    func bridgePayload() -> CKRecordZonePayload {
        ckEncodeZone(self)
    }

    static func fromBridgePayload(_ payload: CKRecordZonePayload) -> CKRecordZone {
        ckDecodeZone(payload)
    }
}
