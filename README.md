# cloudkit

Safe Rust bindings for Apple's [CloudKit](https://developer.apple.com/documentation/cloudkit) framework on macOS.

> **Status:** records, queries, zones, subscriptions, sharing and change fetching reach `CloudKit` through a Swift bridge, with an optional `async` feature. `CKSyncEngine`, the notification models and the operation-configuration types are Rust-only models; [COVERAGE.md](COVERAGE.md) and [COVERAGE_AUDIT.md](COVERAGE_AUDIT.md) list what is bridged and what is modeled.

Requires macOS 13 or later and Rust 1.82 or later.

## Installation

```toml
[dependencies]
cloudkit = "0.4"
```

## Quick start

```rust,no_run
use cloudkit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = CKRecord::new("Task")?;
    record.set_object("title", "Ship v0.4.0");
    record.set_object("done", false);

    let share = CKShare::new_root_record(&record)?;
    println!(
        "record_type={} share_record_type={} participants={}",
        record.record_type(),
        share.share_record().record_type(),
        share.participants().len()
    );
    Ok(())
}
```

## Highlights

- `CKContainer` default/custom containers, account status, user-record lookup, user-identity discovery, and share-participant lookup
- `CKDatabase` record CRUD, first-batch query helpers, fetched query results with resumable cursors, record-zone helpers, and subscription helpers
- Optional `async` feature with executor-agnostic `Future` wrappers for container status/identity calls, database queries/record access, and selected record/database-change operations
- Value-layer Rust wrappers for `CKRecord`, `CKRecordID`, `CKRecordZone`, `CKAsset`, `CKReference`, `CKNotificationInfo`, `CKServerChangeToken`, `CKUserIdentity`, `CKShare`, and `CKShareMetadata`
- Operations for records, queries, database and zone changes, record zones, subscriptions, share metadata / acceptance / access requests, and web-auth tokens
- Exported `CloudKit` constants for container, error, query, record, zone, and share keys
- Headless numbered examples `01`–`15` plus per-area integration tests and `tests/expanded_surface_tests.rs`

## Queries

Build predicates from typed filters. Keys are bound with `%K` and values with `%@`, so user input can never change the predicate's structure:

```rust,no_run
use cloudkit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = CKQuery::new(
        "Task",
        QueryPredicate::and([
            QueryPredicate::compare("owner", ComparisonOperator::Equal, "O'Brien"),
            QueryPredicate::is_in("status", ["open", "blocked"]),
        ]),
    )
    .with_sort_descriptor(CKLocationSortDescriptor::new("location", 59.33, 18.07));
    println!("{}", query.predicate_format()?);
    Ok(())
}
```

`QueryPredicate::format(format, arguments)` accepts a raw `NSPredicate` format string with an argument list. A format that `NSPredicate` cannot parse, or that has too few arguments, returns an error instead of raising an Objective-C exception. `CloudKit` itself only supports a subset of predicate operators (see `CKQuery.h`); unsupported operators fail when the query runs.

Query subscriptions take the same `QueryPredicate`. A subscription fetched from `CloudKit` reports the server's predicate as `QueryPredicate::Format` text, and saving it again reuses the server's archived predicate unchanged.

## Records

Every `CloudKit` field type round-trips: strings, integers, doubles, booleans, bytes, dates, assets, references, locations (`RecordValue::Location`, from `CLLocation`) and lists of them. If `CloudKit` ever returns a value the bridge cannot represent, the call fails instead of dropping the field. Saving a record writes every field it holds and deletes only the keys you removed with `remove_object`; keys the payload does not mention are left alone. `CKRecord::set_parent_reference` rejects references whose action is not `CKReferenceAction::None`, which `CloudKit` would otherwise answer with an exception.

A fetched `CKAsset`'s `file_url()` points into `CloudKit`'s staging area, which the system purges to reclaim disk space. Copy the file somewhere you control if you need it later.

## Timeouts and cancellation

Blocking calls wait for up to the container's timeout, 30 seconds by default. Change it with `CKContainer::with_timeout(Some(duration))`, or pass `None` to wait for `CloudKit`'s own completion. When the timeout expires the bridge cancels the `CloudKit` operation and waits for `CloudKit` to report what finished, so no work keeps running in the background. Operation results then carry `CloudKit`'s per-item outcomes plus a `BridgeTimedOut` operation error; a write that timed out may still have been applied on the server, so fetch the affected records before retrying. Account status, user-record ID and user-identity discovery use `CloudKit` calls that cannot be cancelled; they are read-only and just return `BridgeTimedOut`.

## `CKSyncEngine`

`CKSyncEngine` in this crate is a local simulation. It runs your `CKSyncEngineDelegate` in-process and never contacts `CloudKit`: `simulate_fetch_changes` asks the delegate for fetch options and delivers a `WillFetchChanges` event, and `simulate_send_changes` delivers `WillSendChanges` and returns the delegate's next batch. Fetch and send with the operation types (`CKFetchRecordZoneChangesOperation`, `CKModifyRecordsOperation`) yourself.

## Validation

The bridge is validated with:

```bash
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --all-features
for ex in examples/*.rs; do cargo run --example "$(basename "$ex" .rs)"; done
```

## Entitlements / caveats

`CloudKit` requires an entitled app plus an iCloud account for most server-backed operations.
Unsigned CLI binaries commonly hit `CKErrorMissingEntitlement`, `CKErrorBadContainer`, or `CKErrorNotAuthenticated`. When `CKContainer::default()` cannot be resolved because the current process has no iCloud container entitlement, the crate surfaces a `BridgeDefaultContainerUnavailable` error instead of letting `CloudKit` raise an Objective-C exception. `CKContainer::container(id)` returns `BridgeContainerNotEntitled` when `id` is not in the process's `com.apple.developer.icloud-container-identifiers` entitlement, because `CloudKit` terminates processes that open such a container. Opening a container, adding or removing share participants and changing `allowsAccessRequests` also require the `com.apple.developer.icloud-services` (`CloudKit`), `com.apple.developer.icloud-container-identifiers` and `com.apple.application-identifier` entitlements; without them the crate returns `BridgeCloudKitNotEntitled`, because `CloudKit` would terminate the process with a trap that cannot be caught. Headless tests and examples can therefore report entitlement/account limitations gracefully instead of crashing.

## Examples

Run the defensive framework smoke test with:

```bash
cargo run --example 01_account_status_smoke
```

Other numbered examples cover the expanded logical areas:

- `02_database_zones_and_subscriptions`
- `03_record_value_roundtrip`
- `04_record_id_construction`
- `05_zone_construction`
- `06_subscription_builders`
- `07_operation_builders`
- `08_share_local_construction`
- `09_notification_info_builder`
- `10_query_builder`
- `11_fetched_results_smoke`
- `12_reference_utility`
- `13_server_change_token_bytes`
- `14_asset_file_url`
- `15_user_identity_lookup`

## License

Licensed under either [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
