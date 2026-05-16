use cloudkit::prelude::*;

fn main() {
    let token = CKServerChangeToken::from_archived_data(vec![1_u8, 2, 3, 4]);
    println!("server_change_token_bytes={}", token.archived_data().len());
    println!("✅ server-change-token area OK");
}
