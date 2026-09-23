use cloudkit::prelude::*;

#[test]
fn local_share_creation_exposes_share_record_and_owner_participant() {
    let mut root = CKRecord::new("ShareTests").expect("local record should be constructible");
    root.set_object("title", "share-root");

    let share = CKShare::new_root_record(&root).expect("local share creation should work");

    assert_eq!(share.share_record().record_type(), "cloudkit.share");
    assert!(!share.participants().is_empty());
}

#[test]
fn normalizing_a_share_leaves_unchanged_access_request_settings_alone() {
    let mut root = CKRecord::new("ShareTests").expect("local record should be constructible");
    root.set_object("title", "share-root");

    let share = CKShare::new_root_record(&root)
        .expect("local share creation should work")
        .with_title("Trip")
        .expect("normalizing must not reassign allowsAccessRequests")
        .with_share_type("com.example.trip")
        .expect("normalizing twice should work too");

    assert_eq!(share.title(), Some("Trip"));
    assert_eq!(share.share_type(), Some("com.example.trip"));
}

#[test]
fn share_public_permission_accepts_known_values_and_rejects_unknown_ones() {
    let mut root = CKRecord::new("ShareTests").expect("local record should be constructible");
    root.set_object("title", "share-root");
    let share = CKShare::new_root_record(&root).expect("local share creation should work");

    for permission in [
        CKShareParticipantPermission::Unknown,
        CKShareParticipantPermission::UnknownValue(99),
        CKShareParticipantPermission::UnknownValue(-5),
    ] {
        let error = share
            .clone()
            .with_public_permission(permission)
            .expect_err("unsupported public permissions must be rejected, not abort the process");
        assert_eq!(error.kind(), CloudKitErrorCode::BridgeInvalidArgument);
    }

    let read_only = share
        .with_public_permission(CKShareParticipantPermission::ReadOnly)
        .expect("read-only public permission should be accepted");
    assert_eq!(
        read_only.public_permission(),
        CKShareParticipantPermission::ReadOnly
    );

    let still_read_only = read_only
        .with_title("Trip")
        .expect("normalizing an unchanged public permission should work");
    assert_eq!(
        still_read_only.public_permission(),
        CKShareParticipantPermission::ReadOnly
    );
}
