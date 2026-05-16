# Changelog

## [0.2.0] - 2026-05-16

### Added

- Expanded the Swift bridge and Rust API to cover container, database, record, record ID, zone, subscription, operation, share, notification info, query, fetched results, reference utility, server change token, asset, and user identity logical areas.
- Added Rust wrappers for `CKShare`, `CKShareParticipant`, `CKNotificationInfo`, `CKServerChangeToken`, `CKFetchedQueryResults`, `CKQueryCursor`, `CKUserIdentity`, and `CKPersonNameComponents`.
- Added operation builders for `CKFetchRecordsOperation`, `CKFetchDatabaseChangesOperation`, and `CKFetchRecordZoneChangesOperation` alongside the existing modify/query operations.
- Added record-zone and subscription convenience helpers on `CKDatabase`, plus container helpers for user-identity and share-participant discovery.
- Added numbered headless examples `02` through `15` and per-area integration tests covering each logical area.
- Added `COVERAGE.md` with the audited CloudKit header-to-source coverage map.

### Changed

- Hardened Swift/Rust bridge payload decoding for CloudKit `...ID`/token-style JSON keys used by the bridge.
- Updated crate documentation and README status to reflect the broader v0.2.0 surface.

## [0.1.0] - 2026-05-16

### Added

- Initial public release of `cloudkit-rs` as the `cloudkit` crate.
- `CKContainer` bindings for default/custom containers, account-status lookups, and user-record ID fetches.
- `CKDatabase` bindings for record save/fetch/delete plus first-batch query execution.
- Value-layer Rust types for `CKRecord`, `CKRecordID`, `CKRecordZone`, `CKAsset`, `CKQuery`, and CloudKit record-field values.
- Subscription builders for `CKSubscription`, `CKQuerySubscription`, and `CKRecordZoneSubscription`.
- Batch operation wrappers for `CKModifyRecordsOperation` and `CKQueryOperation`.
- Defensive smoke example `examples/01_account_status_smoke.rs` that handles missing entitlement / no-account environments without crashing.
