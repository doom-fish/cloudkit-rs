use core::ops::{BitOr, BitOrAssign};

use crate::record::CKRecordZoneID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKSubscriptionType {
    Query,
    RecordZone,
    Database,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKNotificationInfo {
    alert_body: Option<String>,
    should_send_content_available: bool,
}

impl Default for CKNotificationInfo {
    fn default() -> Self {
        Self {
            alert_body: None,
            should_send_content_available: true,
        }
    }
}

impl CKNotificationInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alert_body(&self) -> Option<&str> {
        self.alert_body.as_deref()
    }

    pub const fn should_send_content_available(&self) -> bool {
        self.should_send_content_available
    }

    pub fn with_alert_body(mut self, alert_body: impl Into<String>) -> Self {
        self.alert_body = Some(alert_body.into());
        self
    }

    pub fn with_content_available(mut self, should_send_content_available: bool) -> Self {
        self.should_send_content_available = should_send_content_available;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct QuerySubscriptionOptions(u64);

impl QuerySubscriptionOptions {
    pub const FIRES_ON_RECORD_CREATION: Self = Self(1 << 0);
    pub const FIRES_ON_RECORD_UPDATE: Self = Self(1 << 1);
    pub const FIRES_ON_RECORD_DELETION: Self = Self(1 << 2);
    pub const FIRES_ONCE: Self = Self(1 << 3);

    pub const fn bits(self) -> u64 {
        self.0
    }
}

impl BitOr for QuerySubscriptionOptions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for QuerySubscriptionOptions {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKSubscription {
    subscription_id: String,
    subscription_type: CKSubscriptionType,
    notification_info: CKNotificationInfo,
}

impl CKSubscription {
    pub fn new(subscription_id: impl Into<String>, subscription_type: CKSubscriptionType) -> Self {
        Self {
            subscription_id: subscription_id.into(),
            subscription_type,
            notification_info: CKNotificationInfo::default(),
        }
    }

    pub fn subscription_id(&self) -> &str {
        &self.subscription_id
    }

    pub const fn subscription_type(&self) -> CKSubscriptionType {
        self.subscription_type
    }

    pub const fn notification_info(&self) -> &CKNotificationInfo {
        &self.notification_info
    }

    pub fn with_notification_info(mut self, notification_info: CKNotificationInfo) -> Self {
        self.notification_info = notification_info;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKQuerySubscription {
    base: CKSubscription,
    record_type: String,
    predicate_format: String,
    zone_id: Option<CKRecordZoneID>,
    options: QuerySubscriptionOptions,
}

impl CKQuerySubscription {
    pub fn new(
        record_type: impl Into<String>,
        predicate_format: impl Into<String>,
        subscription_id: impl Into<String>,
        options: QuerySubscriptionOptions,
    ) -> Self {
        Self {
            base: CKSubscription::new(subscription_id, CKSubscriptionType::Query),
            record_type: record_type.into(),
            predicate_format: predicate_format.into(),
            zone_id: None,
            options,
        }
    }

    pub const fn base(&self) -> &CKSubscription {
        &self.base
    }

    pub fn record_type(&self) -> &str {
        &self.record_type
    }

    pub fn predicate_format(&self) -> &str {
        &self.predicate_format
    }

    pub const fn zone_id(&self) -> Option<&CKRecordZoneID> {
        self.zone_id.as_ref()
    }

    pub const fn options(&self) -> QuerySubscriptionOptions {
        self.options
    }

    pub fn with_zone_id(mut self, zone_id: CKRecordZoneID) -> Self {
        self.zone_id = Some(zone_id);
        self
    }

    pub fn with_notification_info(mut self, notification_info: CKNotificationInfo) -> Self {
        self.base = self.base.clone().with_notification_info(notification_info);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKRecordZoneSubscription {
    base: CKSubscription,
    zone_id: CKRecordZoneID,
    record_type: Option<String>,
}

impl CKRecordZoneSubscription {
    pub fn new(zone_id: CKRecordZoneID, subscription_id: impl Into<String>) -> Self {
        Self {
            base: CKSubscription::new(subscription_id, CKSubscriptionType::RecordZone),
            zone_id,
            record_type: None,
        }
    }

    pub const fn base(&self) -> &CKSubscription {
        &self.base
    }

    pub const fn zone_id(&self) -> &CKRecordZoneID {
        &self.zone_id
    }

    pub fn record_type(&self) -> Option<&str> {
        self.record_type.as_deref()
    }

    pub fn with_record_type(mut self, record_type: impl Into<String>) -> Self {
        self.record_type = Some(record_type.into());
        self
    }

    pub fn with_notification_info(mut self, notification_info: CKNotificationInfo) -> Self {
        self.base = self.base.clone().with_notification_info(notification_info);
        self
    }
}
