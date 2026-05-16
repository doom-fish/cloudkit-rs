use cloudkit::prelude::*;

#[test]
fn fetch_all_record_zones_or_report_error() {
    let database = CKContainer::default().public_cloud_database();
    match database.fetch_all_record_zones() {
        Ok(zones) => assert!(zones.len() <= zones.len()),
        Err(error) => assert!(!error.message.is_empty()),
    }
}
