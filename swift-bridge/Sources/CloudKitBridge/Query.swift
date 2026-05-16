import CloudKit
import Foundation

func ckEncodeQuery(_ query: CKQuery) -> CKQueryPayload {
    CKQueryPayload(
        recordType: query.recordType,
        predicateFormat: query.predicate.predicateFormat,
        sortDescriptors: (query.sortDescriptors ?? []).map {
            SortDescriptorPayload(key: $0.key ?? "", ascending: $0.ascending)
        }
    )
}

extension CKQuery {
    func bridgePayload() -> CKQueryPayload {
        ckEncodeQuery(self)
    }
}
