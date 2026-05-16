use std::sync::mpsc;
use std::time::Duration;

use cloudkit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== CloudKit.framework smoke ==");

    let container = CKContainer::default();
    let (tx, rx) = mpsc::channel();
    container.account_status_with_completion_handler(move |result| {
        let _ = tx.send(result);
    })?;

    match rx.recv_timeout(Duration::from_secs(30))? {
        Ok(status) => println!("account status: {status}"),
        Err(error) => {
            println!("account status error: {error}");
            if error.is_entitlement_or_account_issue() {
                println!(
                    "note: this is expected for unsigned CLI binaries, missing CloudKit entitlements, or Macs without an iCloud account."
                );
            }
        }
    }

    println!("✅ cloudkit container OK");
    Ok(())
}
