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

#[test]
fn parent_references_must_use_the_none_action() {
    let mut record = CKRecord::new("RecordTests").expect("record creation should work locally");

    let error = record
        .set_parent_reference(CKReference::delete_self(CKRecordID::new("parent")))
        .expect_err("CloudKit raises for non-none parent actions");
    assert_eq!(error.kind(), CloudKitErrorCode::BridgeInvalidArgument);
    assert!(record.parent().is_none());

    record
        .set_parent_reference(CKReference::parent(CKRecordID::new("parent")))
        .expect("a none-action parent is accepted");
    assert_eq!(
        record.parent().map(CKReference::action),
        Some(CKReferenceAction::None)
    );
}

#[test]
fn invalid_record_types_return_errors_instead_of_aborting() {
    for record_type in ["", "a.b"] {
        let error = CKRecord::new(record_type).expect_err("CloudKit rejects the record type");
        assert_eq!(
            error.kind(),
            CloudKitErrorCode::BridgeInvalidArgument,
            "{record_type:?}: {error}"
        );
    }
}

#[test]
fn every_value_type_decodes_into_a_cloudkit_record() {
    let mut record = CKRecord::new("AllValues").expect("record creation should work locally");
    let mut location = RecordLocation::new(59.3293, 18.0686, "2024-05-01T12:00:00.000Z");
    location.altitude = 12.5;
    location.horizontal_accuracy = 5.0;
    record.set_object("text", "hello");
    record.set_object("count", 42_i64);
    record.set_object("ratio", 0.5);
    record.set_object("flag", true);
    record.set_object("blob", vec![1_u8, 2, 3]);
    record.set_object("when", RecordValue::Date("2024-05-01T12:00:00.000Z".into()));
    record.set_object(
        "owner",
        CKReference::delete_self(CKRecordID::new("owner-1")),
    );
    record.set_object("where", location.clone());
    record.set_object(
        "places",
        RecordValue::Array(vec![
            location.into(),
            RecordLocation::new(0.0, 0.0, "1970-01-01T00:00:00.000Z").into(),
        ]),
    );
    record.set_object(
        "attachment",
        CKAsset::new(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")),
    );
    record.set_object("temporary", "gone soon");
    record.remove_object("temporary");

    let share = CKShare::new_root_record(&record)
        .expect("every supported value type should decode in Swift");
    let root = share
        .root_record()
        .expect("a root-record share returns its root record");
    assert_eq!(root.all_keys(), record.all_keys());
    for key in record.all_keys() {
        assert_eq!(root.object(&key), record.object(&key), "field {key}");
    }
    let mut changed_before = record.changed_keys().to_vec();
    let mut changed_after = root.changed_keys().to_vec();
    changed_before.sort();
    changed_after.sort();
    assert_eq!(changed_after, changed_before);
    assert_eq!(
        root.share().map(CKReference::record_id),
        Some(share.share_record().record_id())
    );
}

#[test]
fn invalid_record_contents_return_errors_instead_of_aborting() {
    let cases = [
        ("invalid key", "a.b", RecordValue::from("x")),
        ("reserved key", "recordID", RecordValue::from("x")),
        (
            "nested array",
            "nested",
            RecordValue::Array(vec![RecordValue::Array(vec![])]),
        ),
        (
            "mixed array",
            "mixed",
            RecordValue::Array(vec!["a".into(), RecordValue::Int(1)]),
        ),
        ("asset without a file", "file", CKAsset::new("").into()),
        (
            "unknown reference action",
            "owner",
            CKReference::new(CKRecordID::new("owner"), CKReferenceAction::Unknown(7)).into(),
        ),
        (
            "invalid date",
            "when",
            RecordValue::Date("yesterday".into()),
        ),
    ];

    for (label, key, value) in cases {
        let mut record = CKRecord::new("Invalid").expect("record creation should work locally");
        record.set_object(key, value);
        let error = CKShare::new_root_record(&record).expect_err(label);
        assert!(
            matches!(
                error.kind(),
                CloudKitErrorCode::BridgeInvalidArgument | CloudKitErrorCode::BridgeFailure
            ),
            "{label}: {error}"
        );
    }
}

#[test]
fn invalid_record_and_zone_ids_return_errors_instead_of_aborting() {
    let unnamed = CKRecord::with_zone("Photo", CKRecordZoneID::new("Photos", "__defaultOwner__"));
    let error = CKShare::new_root_record(&unnamed).expect_err("an empty record name is rejected");
    assert!(error.message.contains("recordName"), "{error}");

    let too_long = CKRecord::with_record_id("Photo", CKRecordID::new("a".repeat(256)));
    let error = CKShare::new_root_record(&too_long).expect_err("a 256-character name is rejected");
    assert!(error.message.contains("too long"), "{error}");

    let error = CKShare::new_zone_wide(CKRecordZoneID::new("Zone", ""))
        .expect_err("an empty owner name is rejected");
    assert!(error.message.contains("ownerName"), "{error}");
}
