use cloudkit::prelude::*;

fn main() {
    let info = CKNotificationInfo::new()
        .with_alert_body("A record changed")
        .with_title("CloudKit")
        .with_subtitle("Builder smoke")
        .with_sound_name("ding.aiff")
        .with_should_badge(true)
        .with_mutable_content(true)
        .with_category("cloudkit-demo")
        .with_collapse_id_key("collapseID");

    println!("title={:?} subtitle={:?} mutable={}", info.title(), info.subtitle(), info.should_send_mutable_content());
    println!("✅ notification-info area OK");
}
