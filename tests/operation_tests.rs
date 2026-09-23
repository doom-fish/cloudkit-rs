use cloudkit::prelude::*;

#[test]
fn operation_builders_capture_requested_state() {
    let query = CKQueryOperation::new(CKQuery::match_all("Task")).with_results_limit(25);
    let fetch_records =
        CKFetchRecordsOperation::new(vec![CKRecordID::new("one"), CKRecordID::new("two")]);
    let fetch_db_changes = CKFetchDatabaseChangesOperation::new().with_fetch_all_changes(false);
    let fetch_zone_changes = CKFetchRecordZoneChangesOperation::new(vec![CKRecordZoneID::new(
        "Zone",
        "__defaultOwner__",
    )]);

    assert_eq!(query.results_limit(), Some(25));
    assert!(query.cursor().is_none());
    assert_eq!(fetch_records.record_ids().len(), 2);
    assert!(!fetch_db_changes.fetch_all_changes());
    assert_eq!(fetch_zone_changes.zones().len(), 1);
}

#[test]
fn query_operations_continue_from_a_cursor() {
    let cursor = CKQueryCursor::from_archived_data(vec![4, 5, 6]);
    let operation = CKQueryOperation::new(CKQuery::match_all("Task")).with_cursor(cursor.clone());

    assert_eq!(operation.cursor(), Some(&cursor));

    let error = operation
        .execute_in(&CKContainer::default().private_cloud_database())
        .expect_err("tests run without CloudKit entitlements");
    assert!(error.is_entitlement_or_account_issue(), "{error}");
}
