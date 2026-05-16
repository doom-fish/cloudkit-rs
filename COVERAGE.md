# CloudKit v0.2.0 coverage

This crate audits the practical `CloudKit.framework` surface from the macOS SDK headers and maps each requested logical area to at least one Swift bridge file, one Rust module, one example, and one integration test.

## Audited headers

The v0.2.0 expansion was cross-checked against the current CloudKit SDK headers, including:

- `CKContainer.h`
- `CKDatabase.h`
- `CKRecord.h`
- `CKRecordID.h`
- `CKRecordZone.h`
- `CKReference.h`
- `CKQuery.h`
- `CKSubscription.h`
- `CKOperation.h`
- `CKQueryOperation.h`
- `CKFetchRecordsOperation.h`
- `CKFetchDatabaseChangesOperation.h`
- `CKFetchRecordZoneChangesOperation.h`
- `CKShare.h`
- `CKShareParticipant.h`
- `CKNotification.h`
- `CKServerChangeToken.h`
- `CKUserIdentity.h`
- `CKUserIdentityLookupInfo.h`

## Logical-area map

| Status | Area | Swift bridge | Rust module | Example | Test | Coverage notes |
| --- | --- | --- | --- | --- | --- | --- |
| ✅ implemented | Container | `swift-bridge/Sources/CloudKitBridge/Container.swift` | `src/container.rs` | `examples/01_account_status_smoke.rs` | `tests/container_tests.rs` | Default/custom containers, account status, user record ID, identity discovery, share participant lookup |
| ✅ implemented | Database | `swift-bridge/Sources/CloudKitBridge/Database.swift` | `src/database.rs` | `examples/02_database_zones_and_subscriptions.rs` | `tests/database_tests.rs` | Record CRUD, query convenience, record zones, subscriptions, fetched query results |
| ✅ implemented | Record | `swift-bridge/Sources/CloudKitBridge/Record.swift` | `src/record.rs` | `examples/03_record_value_roundtrip.rs` | `tests/record_tests.rs` | Local record construction, value round-tripping, parent references, metadata |
| ✅ implemented | RecordID | `swift-bridge/Sources/CloudKitBridge/RecordID.swift` | `src/record_id.rs` | `examples/04_record_id_construction.rs` | `tests/record_id_tests.rs` | Record ID and zone ID bridge helpers |
| ✅ implemented | Zone | `swift-bridge/Sources/CloudKitBridge/Zone.swift` | `src/zone.rs` | `examples/05_zone_construction.rs` | `tests/zone_tests.rs` | Record-zone payload helpers, capabilities, encryption scope |
| ✅ implemented | Subscription | `swift-bridge/Sources/CloudKitBridge/Subscription.swift` | `src/subscription.rs` | `examples/06_subscription_builders.rs` | `tests/subscription_tests.rs` | Query, zone, database, and erased subscriptions |
| ✅ implemented | Operation | `swift-bridge/Sources/CloudKitBridge/Operations.swift` | `src/operation.rs` | `examples/07_operation_builders.rs` | `tests/operation_tests.rs` | Modify/query/fetch-records/database-changes/zone-changes operations |
| ✅ implemented | Share | `swift-bridge/Sources/CloudKitBridge/Share.swift` | `src/share.rs` | `examples/08_share_local_construction.rs` | `tests/share_tests.rs` | Local share construction, participant payloads, share normalization |
| ✅ implemented | NotificationInfo | `swift-bridge/Sources/CloudKitBridge/NotificationInfo.swift` | `src/notification_info.rs` | `examples/09_notification_info_builder.rs` | `tests/notification_info_tests.rs` | Alert/body/sound/category/content-available builder surface |
| ✅ implemented | Query | `swift-bridge/Sources/CloudKitBridge/Query.swift` | `src/query.rs` | `examples/10_query_builder.rs` | `tests/query_tests.rs` | Query payload helpers and sort descriptors |
| ✅ implemented | FetchedResults | `swift-bridge/Sources/CloudKitBridge/FetchedResults.swift` | `src/fetched_results.rs` | `examples/11_fetched_results_smoke.rs` | `tests/fetched_results_tests.rs` | Query cursors, record fetch results, database/zone change results |
| ✅ implemented | ReferenceUtility | `swift-bridge/Sources/CloudKitBridge/ReferenceUtility.swift` | `src/reference_utility.rs` | `examples/12_reference_utility.rs` | `tests/reference_utility_tests.rs` | Reference encoding/decoding and delete-self semantics |
| ✅ implemented | ServerChangeToken | `swift-bridge/Sources/CloudKitBridge/ServerChangeToken.swift` | `src/server_change_token.rs` | `examples/13_server_change_token_bytes.rs` | `tests/server_change_token_tests.rs` | Token archiving/round-trip helpers |
| ✅ implemented | Asset | `swift-bridge/Sources/CloudKitBridge/Asset.swift` | `src/asset.rs` | `examples/14_asset_file_url.rs` | `tests/asset_tests.rs` | Asset payload helpers and file-URL round-tripping |
| ✅ implemented | UserIdentity | `swift-bridge/Sources/CloudKitBridge/UserIdentity.swift` | `src/user_identity.rs` | `examples/15_user_identity_lookup.rs` | `tests/user_identity_tests.rs` | Lookup info, person-name components, user identity, share owners |

## Shared bridge foundations

Some cross-cutting payloads and JSON helpers remain centralized in:

- `swift-bridge/Sources/CloudKitBridge/Core.swift`
- `swift-bridge/Sources/CloudKitBridge/Types.swift`
- `src/private.rs`

These files provide the common Codable payloads, JSON codecs, and FFI glue used by the per-area bridge files above.

## Deferred rows

- None in the requested v0.2.0 logical-area expansion.
