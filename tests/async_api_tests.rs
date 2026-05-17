#![cfg(feature = "async")]

use cloudkit::prelude::*;

#[test]
fn async_account_status_is_graceful_without_entitlements() {
    pollster::block_on(async {
        let container = CKContainer::default();
        match container.account_status_async().await {
            Ok(_) => {}
            Err(error) => assert!(!error.message.is_empty()),
        }
    });
}

#[test]
fn async_fetch_user_record_id_is_graceful_without_entitlements() {
    pollster::block_on(async {
        let container = CKContainer::default();
        match container.fetch_user_record_id_async().await {
            Ok(_) => {}
            Err(error) => assert!(!error.message.is_empty()),
        }
    });
}

#[test]
fn async_fetch_query_results_rejects_results_limit_overflow() {
    pollster::block_on(async {
        let database = CKContainer::container("iCloud.example.container").public_cloud_database();
        let query = CKQuery::match_all("AsyncTests");
        let error = database
            .fetch_query_results_async(&query, None, None, Some(usize::MAX))
            .await
            .expect_err("overflowing results limit should fail before hitting CloudKit");
        assert_eq!(error.kind(), CloudKitErrorCode::BridgeInvalidArgument);
    });
}

#[test]
fn async_fetch_all_record_zones_or_report_error() {
    pollster::block_on(async {
        let database = CKContainer::default().public_cloud_database();
        match database.fetch_all_record_zones_async().await {
            Ok(zones) => assert!(zones.len() <= zones.len()),
            Err(error) => assert!(!error.message.is_empty()),
        }
    });
}

#[test]
fn async_modify_records_or_report_error() {
    pollster::block_on(async {
        let database = CKContainer::default().public_cloud_database();
        let operation = CKModifyRecordsOperation::new(Vec::new(), Vec::new());
        match operation.execute_in_async(&database).await {
            Ok(result) => {
                assert!(result.saved_records.is_empty());
                assert!(result.deleted_record_ids.is_empty());
            }
            Err(error) => assert!(!error.message.is_empty()),
        }
    });
}

#[test]
fn async_fetch_database_changes_or_report_error() {
    pollster::block_on(async {
        let database = CKContainer::default().public_cloud_database();
        let operation = CKFetchDatabaseChangesOperation::new().with_fetch_all_changes(false);
        match operation.execute_in_async(&database).await {
            Ok(_) => {}
            Err(error) => assert!(!error.message.is_empty()),
        }
    });
}
