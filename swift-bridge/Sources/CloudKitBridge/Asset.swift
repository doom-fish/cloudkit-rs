import CloudKit
import Foundation

func ckEncodeAsset(_ asset: CKAsset) -> CKAssetPayload {
    CKAssetPayload(fileURL: asset.fileURL?.path)
}

func ckDecodeAsset(_ payload: CKAssetPayload, key: String) throws -> CKAsset {
    guard let fileURL = payload.fileURL, !fileURL.isEmpty else {
        throw ckInvalidArgument("CKAsset for \(key) has no local file URL; CloudKit can only save assets backed by a file")
    }
    return CKAsset(fileURL: URL(fileURLWithPath: fileURL))
}

extension CKAsset {
    func bridgePayload() -> CKAssetPayload {
        ckEncodeAsset(self)
    }
}
