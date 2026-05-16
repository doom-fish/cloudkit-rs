use cloudkit::prelude::*;

#[test]
fn asset_keeps_file_url() {
    let asset = CKAsset::new("/etc/hosts");
    assert_eq!(asset.file_url().to_string_lossy(), "/etc/hosts");
}
