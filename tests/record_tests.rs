use cloudkit::prelude::*;

#[test]
fn record_set_get_remove_and_parent_reference() {
    let mut record = CKRecord::new("RecordTests").expect("record creation should work locally");
    let parent_id = CKRecordID::new("parent");

    record.set_object("name", "hello");
    record.set_object("enabled", true);
    record.set_parent_reference_from_record_id(parent_id.clone());

    assert_eq!(
        record.object("name"),
        Some(&RecordValue::String("hello".into()))
    );
    assert_eq!(
        record
            .parent()
            .map(|reference| reference.record_id().record_name()),
        Some(parent_id.record_name())
    );

    let removed = record.remove_object("enabled");
    assert_eq!(removed, Some(RecordValue::Bool(true)));
    assert!(record.changed_keys().contains(&"enabled".to_string()));
}
