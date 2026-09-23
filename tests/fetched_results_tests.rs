use cloudkit::prelude::*;

#[test]
fn fetch_query_results_returns_data_or_error() {
    let database = CKContainer::default().public_cloud_database();
    let query = CKQuery::match_all("FetchedResultsTests");

    match database.fetch_query_results(&query, None, None, Some(1)) {
        Ok(results) => assert!(results.records.len() <= 1),
        Err(error) => assert!(error.is_entitlement_or_account_issue(), "{error}"),
    }
}

#[test]
fn query_results_resume_from_a_cursor() {
    let database = CKContainer::default().public_cloud_database();
    let cursor = CKQueryCursor::from_archived_data(vec![1, 2, 3]);

    let error = database
        .fetch_query_results_from_cursor(&cursor, Some(vec!["title".into()]), Some(10))
        .expect_err("a garbage cursor or a missing entitlement must fail");
    assert!(
        error.is_entitlement_or_account_issue()
            || error.kind() == CloudKitErrorCode::BridgeInvalidArgument,
        "{error}"
    );
}
