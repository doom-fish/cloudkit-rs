use cloudkit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let asset = CKAsset::new(std::env::current_dir()?.join("Cargo.toml"));
    println!("asset_path={}", asset.file_url().display());
    println!("✅ asset area OK");
    Ok(())
}
