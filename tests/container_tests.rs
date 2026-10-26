use std::time::Duration;

use cloudkit::prelude::*;

#[test]
fn account_status_is_graceful_without_entitlements() {
    let container = CKContainer::default();
    match container.account_status() {
        Ok(status) => assert_ne!(status.to_string(), ""),
        Err(error) => assert!(error.is_entitlement_or_account_issue(), "{error}"),
    }
}

#[test]
fn database_with_scope_matches_requested_scope() {
    let container = CKContainer::container("iCloud.example.container");
    assert_eq!(
        container
            .database_with_scope(CKDatabaseScope::Shared)
            .database_scope(),
        CKDatabaseScope::Shared
    );
}

#[test]
fn unentitled_explicit_containers_return_errors_instead_of_trapping() {
    let container = CKContainer::container("iCloud.fish.doom.cloudkit-rs.tests");

    let error = container
        .account_status()
        .expect_err("test binaries carry no iCloud container entitlement");
    assert_eq!(error.kind(), CloudKitErrorCode::BridgeContainerNotEntitled);
    assert!(error.is_entitlement_or_account_issue());

    let error = container
        .private_cloud_database()
        .fetch_all_record_zones()
        .expect_err("database calls check the entitlement too");
    assert_eq!(error.kind(), CloudKitErrorCode::BridgeContainerNotEntitled);
}

#[test]
fn containers_default_to_a_thirty_second_timeout_that_databases_inherit() {
    let container = CKContainer::default();
    assert_eq!(container.timeout(), Some(Duration::from_secs(30)));

    let container = container.with_timeout(Some(Duration::from_secs(90)));
    assert_eq!(
        container.private_cloud_database().container().timeout(),
        Some(Duration::from_secs(90))
    );
    assert_eq!(container.with_timeout(None).timeout(), None);
}
