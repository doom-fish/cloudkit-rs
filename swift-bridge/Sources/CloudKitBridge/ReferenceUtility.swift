import CloudKit

extension CKRecord.Reference {
    func bridgePayload() -> CKReferencePayload {
        ckEncodeReference(self)
    }

    static func fromBridgePayload(_ payload: CKReferencePayload) -> CKRecord.Reference {
        ckDecodeReference(payload)
    }
}
