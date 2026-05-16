# Changelog

## [0.1.0] - 2026-05-16

### Added

- Initial public release of `cloudkit-rs` as the `cloudkit` crate.
- `CKContainer` bindings for default/custom containers, account-status lookups, and user-record ID fetches.
- `CKDatabase` bindings for record save/fetch/delete plus first-batch query execution.
- Value-layer Rust types for `CKRecord`, `CKRecordID`, `CKRecordZone`, `CKAsset`, `CKQuery`, and CloudKit record-field values.
- Subscription builders for `CKSubscription`, `CKQuerySubscription`, and `CKRecordZoneSubscription`.
- Batch operation wrappers for `CKModifyRecordsOperation` and `CKQueryOperation`.
- Defensive smoke example `examples/01_account_status_smoke.rs` that handles missing entitlement / no-account environments without crashing.
