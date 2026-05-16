use cloudkit::prelude::*;

#[test]
fn record_id_keeps_record_and_zone_names() {
    let zone_id = CKRecordZoneID::new("ZoneA", "OwnerA");
    let record_id = CKRecordID::with_zone("RecordA", zone_id.clone());

    assert_eq!(record_id.record_name(), "RecordA");
    assert_eq!(record_id.zone_id(), &zone_id);
}
