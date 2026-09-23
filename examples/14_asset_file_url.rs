use cloudkit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let asset = CKAsset::new(std::env::current_dir()?.join("Cargo.toml"));
    if let Some(path) = asset.file_url() {
        println!("asset_path={}", path.display());
    }
    println!("✅ asset area OK");
    Ok(())
}
