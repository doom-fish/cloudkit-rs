use cloudkit::prelude::*;

#[test]
fn local_share_creation_exposes_share_record_and_owner_participant() {
    let mut root = CKRecord::new("ShareTests").expect("local record should be constructible");
    root.set_object("title", "share-root");

    let share = CKShare::new_root_record(&root).expect("local share creation should work");

    assert_eq!(share.share_record().record_type(), "cloudkit.share");
    assert!(!share.participants().is_empty());
}
