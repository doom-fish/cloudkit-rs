use cloudkit::prelude::*;

#[test]
fn fetch_all_record_zones_or_report_error() {
    let database = CKContainer::default().public_cloud_database();
    match database.fetch_all_record_zones() {
        Ok(zones) => assert!(
            !zones.is_empty(),
            "CloudKit always returns the default zone"
        ),
        Err(error) => assert!(error.is_entitlement_or_account_issue(), "{error}"),
    }
}
