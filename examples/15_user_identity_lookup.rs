use cloudkit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lookup = CKUserIdentityLookupInfo::with_email_address("developer@example.com");
    let mut root = CKRecord::new("IdentityRoot")?;
    root.set_object("title", "Identity Root");
    let share = CKShare::new_root_record(&root)?;

    println!("lookup_email={:?} owner_has_account={:?}", lookup.email_address(), share.owner().map(|owner| owner.user_identity().has_i_cloud_account()));
    println!("✅ user-identity area OK");
    Ok(())
}
