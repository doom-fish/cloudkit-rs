use cloudkit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut root = CKRecord::new("SharedRoot")?;
    root.set_object("title", "Shared Root");

    let share = CKShare::new_root_record(&root)?;

    println!(
        "share_record_type={} participants={}",
        share.share_record().record_type(),
        share.participants().len()
    );
    println!("✅ share area OK");
    Ok(())
}
