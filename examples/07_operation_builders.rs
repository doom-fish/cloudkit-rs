use cloudkit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let record = CKRecord::new("OperationRecord")?;
    let modify = CKModifyRecordsOperation::new(vec![record], vec![])
        .with_save_policy(CKRecordSavePolicy::ChangedKeys)
        .with_atomic(false);
    let query = CKQueryOperation::new(CKQuery::match_all("OperationRecord")).with_results_limit(1);
    let fetch_records = CKFetchRecordsOperation::new(vec![CKRecordID::new("operation-demo")]);
    let fetch_db_changes = CKFetchDatabaseChangesOperation::new().with_fetch_all_changes(false);
    let fetch_zone_changes = CKFetchRecordZoneChangesOperation::new(vec![CKRecordZoneID::new("OpsZone", "__defaultOwner__")]);

    println!("modify records_to_save={} atomic={}", modify.records_to_save().len(), modify.atomic());
    println!("query limit={:?} fetch_records={} db_changes_fetch_all={} zone_changes={}", query.results_limit(), fetch_records.record_ids().len(), fetch_db_changes.fetch_all_changes(), fetch_zone_changes.zones().len());
    println!("✅ operation area OK");
    Ok(())
}
