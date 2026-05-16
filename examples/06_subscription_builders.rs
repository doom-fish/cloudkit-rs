use cloudkit::prelude::*;

fn main() {
    let notification = CKNotificationInfo::new()
        .with_alert_body("Update available")
        .with_desired_keys(vec!["title".into(), "updatedAt".into()])
        .with_content_available(true);

    let query = CKQuerySubscription::new(
        "Task",
        "TRUEPREDICATE",
        "task-query-sub",
        QuerySubscriptionOptions::FIRES_ON_RECORD_CREATION,
    )
    .with_notification_info(notification.clone());

    let zone = CKRecordZoneSubscription::new(CKRecordZoneID::new("Tasks", "__defaultOwner__"), "task-zone-sub")
        .with_notification_info(notification.clone());
    let database = CKDatabaseSubscription::new("task-db-sub").with_notification_info(notification);

    println!("query={} zone={} database={}", query.base().subscription_id(), zone.base().subscription_id(), database.base().subscription_id());
    println!("✅ subscription area OK");
}
