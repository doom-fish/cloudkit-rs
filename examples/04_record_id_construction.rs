use cloudkit::prelude::*;

fn main() {
    let zone_id = CKRecordZoneID::new("ExampleZone", "__defaultOwner__");
    let record_id = CKRecordID::with_zone("example-record", zone_id.clone());

    println!("record_name={}", record_id.record_name());
    println!(
        "zone={} owner={}",
        zone_id.zone_name(),
        zone_id.owner_name()
    );
    println!("✅ record-id area OK");
}
