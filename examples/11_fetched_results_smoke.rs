use cloudkit::prelude::*;

fn main() {
    let database = CKContainer::default().public_cloud_database();
    let query = CKQuery::match_all("FetchedResultRecord");

    match database.fetch_query_results(&query, None, None, Some(1)) {
        Ok(results) => println!(
            "records={} cursor_present={}",
            results.records.len(),
            results.cursor.is_some()
        ),
        Err(error) => eprintln!("fetch_query_results: {error}"),
    }

    println!("✅ fetched-results area OK");
}
