use cloudkit::prelude::*;

#[test]
fn notification_info_builders_round_trip_fields() {
    let info = CKNotificationInfo::new()
        .with_alert_body("body")
        .with_title("title")
        .with_subtitle("subtitle")
        .with_sound_name("ding.aiff")
        .with_should_badge(true)
        .with_mutable_content(true)
        .with_category("updates")
        .with_collapse_id_key("collapse");

    assert_eq!(info.alert_body(), Some("body"));
    assert_eq!(info.title(), Some("title"));
    assert!(info.should_send_mutable_content());
    assert!(info.should_badge());
}
