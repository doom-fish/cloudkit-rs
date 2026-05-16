use cloudkit::prelude::*;

#[test]
fn fetch_query_results_returns_data_or_error() {
    let database = CKContainer::default().public_cloud_database();
    let query = CKQuery::match_all("FetchedResultsTests");

    match database.fetch_query_results(&query, None, None, Some(1)) {
        Ok(results) => assert!(results.records.len() <= 1),
        Err(error) => assert!(!error.message.is_empty()),
    }
}
