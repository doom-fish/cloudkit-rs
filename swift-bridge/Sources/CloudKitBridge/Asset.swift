import CloudKit
import Foundation

func ckEncodeAsset(_ asset: CKAsset) -> CKAssetPayload {
    CKAssetPayload(fileURL: asset.fileURL?.path ?? "")
}

func ckDecodeAsset(_ payload: CKAssetPayload) -> CKAsset? {
    guard !payload.fileURL.isEmpty else { return nil }
    return CKAsset(fileURL: URL(fileURLWithPath: payload.fileURL))
}

extension CKAsset {
    func bridgePayload() -> CKAssetPayload {
        ckEncodeAsset(self)
    }
}
