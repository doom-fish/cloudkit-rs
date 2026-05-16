use cloudkit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parent_id = CKRecordID::new("parent-record");
    let reference = CKReference::delete_self(parent_id.clone());

    let mut child = CKRecord::new("ChildRecord")?;
    child.set_parent_reference_from_record_id(parent_id);
    child.set_object("parentRef", reference);

    println!("parent={:?} keys={:?}", child.parent().map(|reference| reference.record_id().record_name()), child.all_keys());
    println!("✅ reference-utility area OK");
    Ok(())
}
