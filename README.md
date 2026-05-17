# cloudkit

Safe Rust bindings for Apple's [CloudKit](https://developer.apple.com/documentation/cloudkit) framework on macOS.

> **Status:** v0.3.0 keeps the fully audited CloudKit surface from v0.2.1 and adds an optional `async` feature with `Future`-based container, database, and selected operation wrappers backed by the Swift bridge. See [COVERAGE.md](COVERAGE.md) and [COVERAGE_AUDIT.md](COVERAGE_AUDIT.md) for the audited SDK/header-to-source map.

## Quick start

```rust,no_run
use cloudkit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = CKRecord::new("Task")?;
    record.set_object("title", "Ship v0.3.0");
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
- `CKDatabase` record CRUD, first-batch query helpers, fetched query results, record-zone helpers, and subscription helpers
- Optional `async` feature with executor-agnostic `Future` wrappers for container status/identity calls, database queries/record access, and selected record/database-change operations
- Value-layer Rust wrappers for `CKRecord`, `CKRecordID`, `CKRecordZone`, `CKAsset`, `CKReference`, `CKNotificationInfo`, `CKNotification*`, `CKServerChangeToken`, `CKUserIdentity`, `CKShare`, `CKShareMetadata`, and `CKSyncEngine`
- Operation builders for generic `CKOperation*` configuration plus record-zone, subscription, share metadata / acceptance / access-request, web-auth token, and existing record/query/database-change operations
- Exported `CloudKit` constants for container, error, query, record, zone, and share keys alongside expanded sharing-option helpers and system-sharing observers
- Headless numbered examples `01`–`15` plus per-area integration tests and `tests/expanded_surface_tests.rs`

## Validation

The v0.3.0 bridge is validated with:

```bash
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --all-features
for ex in examples/*.rs; do cargo run --example "$(basename "$ex" .rs)"; done
```

## Entitlements / caveats

`CloudKit` requires an entitled app plus an iCloud account for most server-backed operations.
Unsigned CLI binaries commonly hit `CKErrorMissingEntitlement`, `CKErrorBadContainer`, or `CKErrorNotAuthenticated`. When `CKContainer::default()` cannot be resolved because the current process has no iCloud container entitlement, the crate surfaces a bridge error instead of letting `CloudKit` raise an Objective-C exception. Headless tests and examples can therefore report entitlement/account limitations gracefully instead of crashing.

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
