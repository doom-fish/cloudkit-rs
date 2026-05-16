use core::ops::{BitOr, BitOrAssign};

use crate::notification_info::CKNotificationInfo;
use crate::private::{CKNotificationInfoPayload, CKSubscriptionPayload, CKSubscriptionPayloadKind};
use crate::record::CKRecordZoneID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKSubscriptionType {
    Query,
    RecordZone,
    Database,
    Unknown(i32),
}

impl CKSubscriptionType {
    pub(crate) const fn to_raw(self) -> i32 {
        match self {
            Self::Query => 1,
            Self::RecordZone => 2,
            Self::Database => 3,
            Self::Unknown(raw) => raw,
        }
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

    pub fn notification_info(&self) -> &CKNotificationInfo {
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

    pub(crate) fn from_payload(payload: CKSubscriptionPayload) -> Self {
        let mut subscription = Self::new(
            payload.record_type.unwrap_or_default(),
            payload.predicate_format.unwrap_or_else(|| "TRUEPREDICATE".into()),
            payload.subscription_id,
            QuerySubscriptionOptions(payload.query_subscription_options.unwrap_or_default()),
        );
        if let Some(zone_id) = payload.zone_id {
            subscription.zone_id = Some(CKRecordZoneID::from_payload(zone_id));
        }
        if let Some(notification_info) = payload.notification_info {
            subscription.base.notification_info = CKNotificationInfo::from_payload(notification_info);
        }
        subscription
    }

    pub(crate) fn to_payload(&self) -> CKSubscriptionPayload {
        CKSubscriptionPayload {
            kind: CKSubscriptionPayloadKind::Query,
            subscription_id: self.base.subscription_id.clone(),
            subscription_type: self.base.subscription_type.to_raw(),
            notification_info: Some(self.base.notification_info.to_payload()),
            record_type: Some(self.record_type.clone()),
            predicate_format: Some(self.predicate_format.clone()),
            zone_id: self.zone_id.as_ref().map(CKRecordZoneID::to_payload),
            query_subscription_options: Some(self.options.bits()),
        }
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

    pub(crate) fn from_payload(payload: CKSubscriptionPayload) -> Self {
        let mut subscription = Self::new(
            payload
                .zone_id
                .map_or_else(CKRecordZoneID::default_zone, CKRecordZoneID::from_payload),
            payload.subscription_id,
        );
        subscription.record_type = payload.record_type;
        if let Some(notification_info) = payload.notification_info {
            subscription.base.notification_info = CKNotificationInfo::from_payload(notification_info);
        }
        subscription
    }

    pub(crate) fn to_payload(&self) -> CKSubscriptionPayload {
        CKSubscriptionPayload {
            kind: CKSubscriptionPayloadKind::RecordZone,
            subscription_id: self.base.subscription_id.clone(),
            subscription_type: self.base.subscription_type.to_raw(),
            notification_info: Some(self.base.notification_info.to_payload()),
            record_type: self.record_type.clone(),
            predicate_format: None,
            zone_id: Some(self.zone_id.to_payload()),
            query_subscription_options: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKDatabaseSubscription {
    base: CKSubscription,
    record_type: Option<String>,
}

impl CKDatabaseSubscription {
    pub fn new(subscription_id: impl Into<String>) -> Self {
        Self {
            base: CKSubscription::new(subscription_id, CKSubscriptionType::Database),
            record_type: None,
        }
    }

    pub const fn base(&self) -> &CKSubscription {
        &self.base
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

    pub(crate) fn from_payload(payload: CKSubscriptionPayload) -> Self {
        let mut subscription = Self::new(payload.subscription_id);
        subscription.record_type = payload.record_type;
        if let Some(notification_info) = payload.notification_info {
            subscription.base.notification_info = CKNotificationInfo::from_payload(notification_info);
        }
        subscription
    }

    pub(crate) fn to_payload(&self) -> CKSubscriptionPayload {
        CKSubscriptionPayload {
            kind: CKSubscriptionPayloadKind::Database,
            subscription_id: self.base.subscription_id.clone(),
            subscription_type: self.base.subscription_type.to_raw(),
            notification_info: Some(self.base.notification_info.to_payload()),
            record_type: self.record_type.clone(),
            predicate_format: None,
            zone_id: None,
            query_subscription_options: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKAnySubscription {
    Query(CKQuerySubscription),
    RecordZone(CKRecordZoneSubscription),
    Database(CKDatabaseSubscription),
}

impl CKAnySubscription {
    pub fn subscription_id(&self) -> &str {
        match self {
            Self::Query(subscription) => subscription.base().subscription_id(),
            Self::RecordZone(subscription) => subscription.base().subscription_id(),
            Self::Database(subscription) => subscription.base().subscription_id(),
        }
    }

    pub fn notification_info(&self) -> &CKNotificationInfo {
        match self {
            Self::Query(subscription) => subscription.base().notification_info(),
            Self::RecordZone(subscription) => subscription.base().notification_info(),
            Self::Database(subscription) => subscription.base().notification_info(),
        }
    }

    pub const fn subscription_type(&self) -> CKSubscriptionType {
        match self {
            Self::Query(subscription) => subscription.base().subscription_type(),
            Self::RecordZone(subscription) => subscription.base().subscription_type(),
            Self::Database(subscription) => subscription.base().subscription_type(),
        }
    }

    pub(crate) fn from_payload(payload: CKSubscriptionPayload) -> Self {
        match payload.kind {
            CKSubscriptionPayloadKind::Query => Self::Query(CKQuerySubscription::from_payload(payload)),
            CKSubscriptionPayloadKind::RecordZone => {
                Self::RecordZone(CKRecordZoneSubscription::from_payload(payload))
            }
            CKSubscriptionPayloadKind::Database => {
                Self::Database(CKDatabaseSubscription::from_payload(payload))
            }
        }
    }

    pub(crate) fn to_payload(&self) -> CKSubscriptionPayload {
        match self {
            Self::Query(subscription) => subscription.to_payload(),
            Self::RecordZone(subscription) => subscription.to_payload(),
            Self::Database(subscription) => subscription.to_payload(),
        }
    }
}

impl From<CKQuerySubscription> for CKAnySubscription {
    fn from(value: CKQuerySubscription) -> Self {
        Self::Query(value)
    }
}

impl From<CKRecordZoneSubscription> for CKAnySubscription {
    fn from(value: CKRecordZoneSubscription) -> Self {
        Self::RecordZone(value)
    }
}

impl From<CKDatabaseSubscription> for CKAnySubscription {
    fn from(value: CKDatabaseSubscription) -> Self {
        Self::Database(value)
    }
}

impl From<CKNotificationInfoPayload> for CKNotificationInfo {
    fn from(value: CKNotificationInfoPayload) -> Self {
        CKNotificationInfo::from_payload(value)
    }
}
