use cloudkit::prelude::*;

#[test]
fn server_change_token_preserves_archived_bytes() {
    let token = CKServerChangeToken::from_archived_data(vec![1_u8, 2, 3]);
    assert_eq!(token.archived_data(), &[1, 2, 3]);
}
