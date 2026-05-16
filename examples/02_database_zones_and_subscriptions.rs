use cloudkit::prelude::*;

fn main() {
    let database = CKContainer::default().public_cloud_database();

    match database.fetch_all_record_zones() {
        Ok(zones) => println!("record zones: {}", zones.len()),
        Err(error) => eprintln!("fetch_all_record_zones: {error}"),
    }

    match database.fetch_all_subscriptions() {
        Ok(subscriptions) => println!("subscriptions: {}", subscriptions.len()),
        Err(error) => eprintln!("fetch_all_subscriptions: {error}"),
    }

    println!("✅ database area OK");
}
