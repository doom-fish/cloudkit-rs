# cloudkit

Safe Rust bindings for Apple's [CloudKit](https://developer.apple.com/documentation/cloudkit) framework on macOS.

> **Status:** v0.1.0 covers the practical CloudKit container/database surface for account checks, record CRUD, query execution, record values/assets, record IDs/zones, subscription builders, and the two most useful batch operations (`CKModifyRecordsOperation`, `CKQueryOperation`).

## Quick start

```rust,no_run
use cloudkit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let container = CKContainer::default();
    match container.account_status() {
        Ok(status) => println!("CloudKit account status: {status:?}"),
        Err(error) if error.is_entitlement_or_account_issue() => {
            eprintln!("CloudKit unavailable in this process: {error}");
        }
        Err(error) => return Err(error.into()),
    }

    let database = container.public_cloud_database();
    let mut record = CKRecord::new("SmokeRecord")?;
    record.set_object("title", "doom-fish");
    record.set_object("count", 1_i64);

    let saved = database.save_record(&record)?;
    println!("saved record {}", saved.record_id().record_name());
    Ok(())
}
```

## Highlights

- `CKContainer::default`, `CKContainer::container`, `account_status`, and `fetch_user_record_id`
- `CKDatabase::{save_record, fetch_record, delete_record, perform_query}`
- `CKRecord`, `CKRecordID`, `CKRecordZone`, `CKAsset`, `RecordValue`
- `CKQuery` with sort descriptors
- `CKSubscription`, `CKQuerySubscription`, and `CKRecordZoneSubscription` builder types
- `CKModifyRecordsOperation` and `CKQueryOperation`
- Async completion-handler APIs bridged to Rust callbacks for account status, user-record lookup, and query execution

## Entitlements / caveats

`CloudKit` requires an entitled app plus an iCloud account for most server-backed operations.
Unsigned CLI binaries commonly hit `CKErrorMissingEntitlement`, `CKErrorBadContainer`, or `CKErrorNotAuthenticated`. When `CKContainer::default()` cannot be resolved because the current process has no iCloud container entitlement, the crate now surfaces a bridge error instead of letting `CloudKit` raise an Objective-C exception. Smoke tests and applications can therefore report those cases gracefully instead of crashing.

## Smoke example

Run the defensive framework smoke test with:

```bash
cargo run --example 01_account_status_smoke
```

It calls `CKContainer::default().account_status_with_completion_handler(...)`, prints the resulting status (or the entitlement/account error, including a missing-default-container bridge error on unsigned CLIs), and always prints `✅ cloudkit container OK` once the framework call completes.

## License

Licensed under either [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
