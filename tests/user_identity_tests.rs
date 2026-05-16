use cloudkit::prelude::*;

#[test]
fn user_identity_lookup_info_and_share_owner_are_available() {
    let lookup = CKUserIdentityLookupInfo::with_email_address("developer@example.com");
    assert_eq!(lookup.email_address(), Some("developer@example.com"));

    let root = CKRecord::new("IdentityTests").expect("local record should work");
    let share = CKShare::new_root_record(&root).expect("local share should work");
    let owner = share.owner().expect("shares should expose an owner participant");
    assert!(owner.user_identity().has_i_cloud_account());
}
