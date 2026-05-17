use cloudkit::prelude::*;

fn main() {
    let zone = CKRecordZone::new("ProjectZone");
    println!(
        "zone={} capabilities={}",
        zone.zone_id().zone_name(),
        zone.capabilities().bits()
    );
    println!("✅ zone area OK");
}
