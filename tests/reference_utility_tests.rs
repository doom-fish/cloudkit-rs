use cloudkit::prelude::*;

#[test]
fn reference_helpers_keep_record_ids_and_actions() {
    let record_id = CKRecordID::new("parent");
    let delete_self = CKReference::delete_self(record_id.clone());
    let parent = CKReference::parent(record_id.clone());

    assert_eq!(delete_self.record_id(), &record_id);
    assert_eq!(delete_self.action(), CKReferenceAction::DeleteSelf);
    assert_eq!(parent.action(), CKReferenceAction::None);
}
