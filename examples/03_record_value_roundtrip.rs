use cloudkit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut record = CKRecord::new("ExampleRecord")?;
    record.set_object("title", "cloudkit-rs");
    record.set_object("count", 2_i64);
    record.set_object("enabled", true);
    record.set_object("bytes", vec![1_u8, 2, 3]);

    println!(
        "record_type={} keys={:?}",
        record.record_type(),
        record.all_keys()
    );
    println!("changed_keys={:?}", record.changed_keys());
    println!("✅ record area OK");
    Ok(())
}
