use cloudkit::prelude::*;

#[test]
fn account_status_is_graceful_without_entitlements() {
    let container = CKContainer::default();
    match container.account_status() {
        Ok(_) => {}
        Err(error) => assert!(!error.message.is_empty()),
    }
}

#[test]
fn database_with_scope_matches_requested_scope() {
    let container = CKContainer::container("iCloud.example.container");
    assert_eq!(container.database_with_scope(CKDatabaseScope::Shared).database_scope(), CKDatabaseScope::Shared);
}
