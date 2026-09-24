# Changelog

All notable changes to `cloudkit` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - Unreleased

### Security

- `CKQuery` and `CKQuerySubscription` no longer take raw predicate format
  strings. Predicates are built from the typed `QueryPredicate` with `%K`
  keys and `%@` values, so caller data cannot change a predicate's structure.
  The raw escape hatch, `QueryPredicate::format(format, arguments)`, runs
  inside an Objective-C `@try` shim and returns an error for unparsable
  formats or missing arguments instead of aborting the process.

### Fixed

- Records no longer lose data in a fetch, modify, save cycle. Encoding handles
  every CloudKit value type, including `CLLocation`, and fails loudly on a
  value it cannot represent instead of skipping the field. Applying a record
  payload no longer clears keys the payload does not mention; it removes only
  keys that were removed with `CKRecord::remove_object`, which previously did
  not delete the field on the server. Per-record encode failures inside
  operations are reported as per-record errors, and changed records that
  cannot be encoded fail a zone-changes fetch instead of vanishing.
- `CKAsset` values could not cross the bridge at all: Rust sent and expected
  `fileUrl` while Swift used `fileURL`. Assets now round-trip, and a fetched
  asset without a local file is represented as `file_url() == None`.
- Blocking calls cancel the CloudKit operation when their timeout expires and
  then wait for CloudKit's final per-item results, so a timed-out save can no
  longer keep running and commit after the caller received an error. Every
  sync call now runs as a cancellable operation (record, zone and
  subscription CRUD, queries, fetches, share metadata, share acceptance,
  share access requests, share-participant lookup, web-auth tokens).
- `CKLocationSortDescriptor` can be attached to a `CKQuery`.
- Query cursors are resumable: `CKDatabase::fetch_query_results_from_cursor`,
  `fetch_query_results_from_cursor_async` and `CKQueryOperation::with_cursor`
  continue a query, and `QueryOperationResult::cursor` returns the cursor.
- Inputs that made CloudKit raise an Objective-C exception, which aborted the
  process, now return errors: parent references with a non-`None` action,
  empty or over-long record names and zone names, empty owner names, invalid
  or empty record types, invalid or reserved field keys, nested or mixed-type
  lists, query subscriptions without a firing option, empty subscription IDs
  and record types, and unknown `CKReferenceAction` values.
- `CKContainer::container(id)` returns `BridgeContainerNotEntitled` instead of
  letting CloudKit terminate the process when `id` is not in the process's
  iCloud container entitlement.
- The bridge no longer traps: the `as! CKDatabaseSubscription` cast became an
  error for unknown subscription classes, `Int32(error.code)` conversions no
  longer trap or report success for a zero error code, and duplicate zone IDs
  in `CKFetchRecordZoneChangesOperation` no longer crash
  `Dictionary(uniqueKeysWithValues:)`.
- The fallback error JSON used when an error payload cannot be encoded is a
  constant, properly escaped document instead of unescaped string
  interpolation.
- The async query path fails loudly instead of returning an empty list when a
  record cannot be encoded.
- Normalizing a share (`CKShare::with_title` and the other `with_*` setters)
  reassigned `allowsAccessRequests` on every call, which terminated
  unentitled processes and raises an exception for participants other than
  the owner. The bridge now sets it only when the value changes.
- `CKShare::with_public_permission(CKShareParticipantPermission::Unknown)`
  made CloudKit raise an exception that aborted the process, and other
  unknown raw values were stored unchecked. Both now return
  `BridgeInvalidArgument`, and the bridge sets `publicPermission` only when
  it changes.
- Adding or removing share participants and changing `allowsAccessRequests`
  made CloudKit terminate unentitled processes with a trap that cannot be
  caught. The bridge now checks the `com.apple.developer.icloud-services`
  (CloudKit), `com.apple.developer.icloud-container-identifiers` and
  `com.apple.application-identifier` entitlements first and returns
  `BridgeCloudKitNotEntitled`; opening a container requires the same
  entitlements. In entitled processes these calls run inside the `@try` shim,
  so a CloudKit exception becomes an error.
- Saving a record re-set every field, so CloudKit wrote unchanged fields
  again and re-uploaded unchanged assets from their staged files, and saving
  a fetched record whose asset had no local file failed. Saves now send only
  the keys set or removed since the record was fetched or saved
  (`CKRecord::changed_keys`), as a native `CKRecord` does, unless the
  operation uses `CKRecordSavePolicy::AllKeys`. The saved record that comes back
  holds every field and no pending changes. A share's title, thumbnail and
  type are tracked the same way instead of being re-set on every normalize
  and save.
- `CKShare::new_root_record` returned the root record without the share
  reference CloudKit sets on it, so saving the share with that root record
  dropped the link. `root_record()` now carries the reference.

### Changed

- **Breaking:** `CKQuery::new` and `CKQuerySubscription::new` take a
  `QueryPredicate` instead of a format string. `CKQuery::predicate_format`
  now builds the `NSPredicate` through the bridge and returns
  `Result<String, CloudKitError>`. `CKQuerySubscription::predicate_format`
  was replaced by `predicate()`; a fetched subscription reports the server's
  predicate as `QueryPredicate::Format` text and re-saving it reuses the
  archived server predicate.
- **Breaking:** `CKQuery::sort_descriptors` returns `&[QuerySortDescriptor]`,
  which holds either a `SortDescriptor` or a `CKLocationSortDescriptor`;
  `with_sort_descriptor` and `set_sort_descriptors` accept either kind.
- **Breaking:** `CKQuery`, `CKQueryOperation`, `CKQuerySubscription` and
  `CKAnySubscription` no longer implement `Eq`/`Hash`, because predicates hold
  floating-point values.
- **Breaking:** `CKAsset::file_url` returns `Option<&Path>`.
- **Breaking:** `CKRecord::set_parent_reference` returns
  `Result<(), CloudKitError>` and rejects actions other than
  `CKReferenceAction::None`.
- **Breaking:** `CKSyncEngine` is documented as the local simulation it is.
  `fetch_changes` and `send_changes` were renamed to `simulate_fetch_changes`
  and `simulate_send_changes`; they never contacted CloudKit.
- **Breaking:** `RecordValue` gained a `Location` variant (the enum was already
  `#[non_exhaustive]`), and `QueryOperationResult` gained a `cursor` field.
- **Breaking:** saves write only changed keys. Code that relied on a save
  re-writing every local field must run a `CKModifyRecordsOperation` with
  `CKRecordSavePolicy::AllKeys`.
- **Breaking:** `CKShare::remove_participant` returns `BridgeInvalidArgument`
  for the share's owner instead of silently keeping the owner,
  `with_allows_access_requests` returns an error before macOS 26 instead of
  dropping the value, and a participant without archived data is rejected
  instead of being replaced by a new one-time URL participant.
- Blocking calls default to a 30-second timeout that is now configurable with
  `CKContainer::with_timeout`; `None` waits for CloudKit to finish.
- `rust-version` is now 1.82 and `doom-fish-utils` must be `>=0.4.1, <0.5`.
- COVERAGE.md and COVERAGE_AUDIT*.md no longer count Rust-only models as
  VERIFIED: `CKSyncEngine*`, the `CKNotification` models, the
  `CKOperation*` configuration types and the sharing-option models are listed
  as MODELED, which puts bridged coverage at 77 of 134 non-exempt symbols.

### Added

- `QueryPredicate`, `ComparisonOperator` and `QuerySortDescriptor`.
- `RecordLocation`, whose timestamp is required because `CLLocation` always
  has one, and `RecordValue::Location`.
- `CKContainer::with_timeout`, `CKContainer::timeout` and
  `container::DEFAULT_TIMEOUT`.
- `CKQueryOperation::cursor`/`with_cursor`,
  `CKDatabase::fetch_query_results_from_cursor` and
  `fetch_query_results_from_cursor_async`.
- `CloudKitErrorCode::BridgeContainerNotEntitled` and
  `CloudKitErrorCode::BridgeCloudKitNotEntitled`.

### Removed

- **Breaking:** `CKSyncEngine::cancel_operations`. The simulation never starts
  CloudKit operations, so there was nothing to cancel.

## [0.3.8] - 2026-06-06

- Null-checked the JSON pointers in the CloudKit completion trampolines and
  removed the vestigial Swift bridge C header.

## [0.3.7] - 2026-05-20

- Migrated local `take_string` body to call `doom_fish_utils::ffi_string::take_owned_cstring_c`. Centralises the duplicated FFI take-string pattern fleet-wide. No public API change.

## [0.3.6] - 2026-05-20

- Added in-`src/` unit tests across src/error.rs, src/notification_info.rs, and src/record.rs (Tier 2 quality polish), providing fast `cargo test --lib` fail-fast signal alongside the existing integration tests under `tests/`.

## [0.3.5] - 2026-05-20

- Widen `doom-fish-utils` dependency bound to `<0.4` so the 0.3.x SPSC-ring release resolves cleanly. No source changes.

## [0.3.4] - 2026-05-19

- Bump MSRV from 1.70 to 1.76 to match fleet baseline.

## [0.3.3] - 2026-05-18

### Changed

- Added one-line `///` docs across the public CloudKit wrapper surface, including modules, constants, sync/share/query types, and the optional async API, bringing the crate to full `missing_docs` coverage under `cargo rustdoc --all-features -- -D missing-docs`.

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
