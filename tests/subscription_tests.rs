use cloudkit::prelude::*;

#[test]
fn subscriptions_build_into_any_subscription() {
    let notification = CKNotificationInfo::new().with_alert_body("updated");
    let predicate = QueryPredicate::compare("status", ComparisonOperator::Equal, "open");
    let query = CKQuerySubscription::new(
        "Task",
        predicate.clone(),
        "query-sub",
        QuerySubscriptionOptions::FIRES_ON_RECORD_UPDATE,
    )
    .with_notification_info(notification.clone());
    let zone =
        CKRecordZoneSubscription::new(CKRecordZoneID::new("Tasks", "__defaultOwner__"), "zone-sub")
            .with_notification_info(notification.clone());
    let database = CKDatabaseSubscription::new("db-sub").with_notification_info(notification);

    assert_eq!(query.predicate(), &predicate);

    let subscriptions = [
        CKAnySubscription::from(query),
        CKAnySubscription::from(zone),
        CKAnySubscription::from(database),
    ];
    assert_eq!(subscriptions.len(), 3);
    assert_eq!(subscriptions[0].subscription_id(), "query-sub");
    assert_eq!(
        subscriptions[0].subscription_type(),
        CKSubscriptionType::Query
    );
}
