# Changelog

## [0.3.2] - 2026-05-18

### Changed

- Added `Debug` implementations for the async `*Future` public wrapper types so every public struct in the crate now exposes `Debug`.

## [0.3.1] - 2026-05-17

### Fixed

- Wrapped all five `extern "C"` bridge callbacks (`status_callback`, `json_callback`, `query_callback_trampoline`, `account_status_trampoline`, `record_id_trampoline`) in `doom_fish_utils::panic_safe::catch_user_panic` — previously, a panic inside any callback would unwind across the FFI boundary (undefined behaviour).
- Renamed misleading variable `payload` to `json_str` in `json_callback` success path.
- Added `SAFETY:` comments to every `unsafe` block in `src/database.rs`, `src/operation.rs`, `src/operation_support.rs`, `src/async_api.rs`, and `src/container.rs`.
- Added `# Safety` doc sections to the unsafe helpers in `src/private.rs`.
- Added one-line `///` doc comments to all public `*Future` types in `src/async_api.rs`.
- Widened `doom-fish-utils` version range to `>=0.1, <0.3` to allow the next minor release.

## [0.3.0] - 2026-05-17

### Added

- Added an optional `async` cargo feature with `Future`-based wrappers for container account/user identity APIs, database record/query helpers, and selected record/database-change operations.
- Added callback-based Swift thunks in `swift-bridge/Sources/CloudKitBridge/Async.swift` for the Tier 1 async CloudKit surface.
- Added async smoke tests gated behind `--features async` plus `pollster` and `doom-fish-utils` support dependencies.

### Changed

- Exported `CKApplicationPermissionStatus` and refreshed the README validation guidance to cover the async feature.

## [0.2.1] - 2026-05-17

### Added

- Added exported CloudKit constants in `src/constants.rs` for container, error, query, record, zone, and share keys verified against the current macOS SDK.
- Added pure-Rust notification wrappers in `src/notification.rs` for `CKNotification`, `CKQueryNotification`, `CKRecordZoneNotification`, `CKDatabaseNotification`, and related ID/reason/type enums.
- Added generic operation/configuration wrappers in `src/operation_support.rs` covering `CKOperation*`, record-zone/subscription operation subclasses, share metadata/acceptance/access-request operations, and web-auth token fetching.
- Added sharing-option, requester, blocked-identity, share-metadata, and system-sharing observer models in `src/share.rs`.
- Added the `CKSyncEngine` family in `src/sync_engine.rs`, including configuration, delegate, state serialization, pending changes, change batches, and event models.
- Added `tests/expanded_surface_tests.rs` to smoke-test the newly closed symbol families.

### Changed

- Updated the Swift bridge to modern CloudKit Swift API names for share metadata fetching/acceptance, share access requests, and web-auth token operations.
- Closed the remaining `COVERAGE_AUDIT.md` gaps and raised the audited non-exempt SDK coverage to 100%.
- Refreshed README/COVERAGE documentation for the expanded v0.2.1 surface.

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
