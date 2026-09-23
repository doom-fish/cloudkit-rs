import CloudKit

extension CKRecord.Reference {
    func bridgePayload() -> CKReferencePayload {
        ckEncodeReference(self)
    }

    static func fromBridgePayload(_ payload: CKReferencePayload) throws -> CKRecord.Reference {
        try ckDecodeReference(payload)
    }
}
