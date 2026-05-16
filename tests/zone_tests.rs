use cloudkit::prelude::*;

#[test]
fn zone_defaults_are_stable() {
    let zone = CKRecordZone::default_zone();
    assert_eq!(zone.zone_id().zone_name(), "_defaultZone");
    assert_eq!(zone.capabilities().bits(), 0);
}
