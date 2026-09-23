use cloudkit::prelude::*;

#[test]
fn asset_keeps_file_url() {
    let asset = CKAsset::new("/etc/hosts");
    assert_eq!(asset.file_url(), Some(std::path::Path::new("/etc/hosts")));
}
