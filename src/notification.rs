use std::collections::BTreeMap;

use crate::database::CKDatabaseScope;
use crate::record::{CKRecordID, CKRecordZoneID, RecordValue};

/// Wraps `CKNotificationID`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct CKNotificationID {
    archived_data: Vec<u8>,
}

impl CKNotificationID {
    /// Creates a wrapper mirroring `CKNotificationID`.
    pub fn new(archived_data: Vec<u8>) -> Self {
        Self { archived_data }
    }

    /// Mirrors archived data stored by `CKNotificationID`.
    pub fn archived_data(&self) -> &[u8] {
        &self.archived_data
    }
}

/// Mirrors `CKNotificationType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKNotificationType {
    /// Mirrors `CKNotificationType.query`.
    Query,
    /// Mirrors `CKNotificationType.recordZone`.
    RecordZone,
    /// Mirrors `CKNotificationType.readNotification`.
    ReadNotification,
    /// Mirrors `CKNotificationType.database`.
    Database,
    /// Mirrors `CKNotificationType.unknown`.
    Unknown(i32),
}

impl CKNotificationType {
    /// Decodes the raw value used by `CKNotificationType`.
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Query,
            2 => Self::RecordZone,
            3 => Self::ReadNotification,
            4 => Self::Database,
            other => Self::Unknown(other),
        }
    }

    /// Encodes the raw value used by `CKNotificationType`.
    pub const fn to_raw(self) -> i32 {
        match self {
            Self::Query => 1,
            Self::RecordZone => 2,
            Self::ReadNotification => 3,
            Self::Database => 4,
            Self::Unknown(raw) => raw,
        }
    }
}

/// Mirrors `CKQueryNotificationReason`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKQueryNotificationReason {
    /// Mirrors `CKQueryNotificationReason.recordCreated`.
    RecordCreated,
    /// Mirrors `CKQueryNotificationReason.recordUpdated`.
    RecordUpdated,
    /// Mirrors `CKQueryNotificationReason.recordDeleted`.
    RecordDeleted,
    /// Mirrors `CKQueryNotificationReason.unknown`.
    Unknown(i32),
}

impl CKQueryNotificationReason {
    /// Decodes the raw value used by `CKQueryNotificationReason`.
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::RecordCreated,
            2 => Self::RecordUpdated,
            3 => Self::RecordDeleted,
            other => Self::Unknown(other),
        }
    }

    /// Encodes the raw value used by `CKQueryNotificationReason`.
    pub const fn to_raw(self) -> i32 {
        match self {
            Self::RecordCreated => 1,
            Self::RecordUpdated => 2,
            Self::RecordDeleted => 3,
            Self::Unknown(raw) => raw,
        }
    }
}

/// Wraps `CKNotification`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKNotification {
    notification_type: CKNotificationType,
    notification_id: Option<CKNotificationID>,
    container_identifier: Option<String>,
    subscription_owner_user_record_id: Option<CKRecordID>,
    is_pruned: bool,
    subscription_id: Option<String>,
}

impl CKNotification {
    /// Creates a wrapper mirroring `CKNotification`.
    pub fn new(notification_type: CKNotificationType) -> Self {
        Self {
            notification_type,
            notification_id: None,
            container_identifier: None,
            subscription_owner_user_record_id: None,
            is_pruned: false,
            subscription_id: None,
        }
    }

    /// Mirrors `CKNotification.notificationType`.
    pub const fn notification_type(&self) -> CKNotificationType {
        self.notification_type
    }

    /// Mirrors `CKNotification.notificationID`.
    pub const fn notification_id(&self) -> Option<&CKNotificationID> {
        self.notification_id.as_ref()
    }

    /// Mirrors `CKNotification.containerIdentifier`.
    pub fn container_identifier(&self) -> Option<&str> {
        self.container_identifier.as_deref()
    }

    /// Mirrors `CKNotification.subscriptionOwnerUserRecordID`.
    pub const fn subscription_owner_user_record_id(&self) -> Option<&CKRecordID> {
        self.subscription_owner_user_record_id.as_ref()
    }

    /// Mirrors `CKNotification.isPruned`.
    pub const fn is_pruned(&self) -> bool {
        self.is_pruned
    }

    /// Mirrors `CKNotification.subscriptionID`.
    pub fn subscription_id(&self) -> Option<&str> {
        self.subscription_id.as_deref()
    }

    /// Sets the value mirroring `CKNotification.notificationID`.
    pub fn with_notification_id(mut self, notification_id: CKNotificationID) -> Self {
        self.notification_id = Some(notification_id);
        self
    }

    /// Sets the value mirroring `CKNotification.containerIdentifier`.
    pub fn with_container_identifier(mut self, container_identifier: impl Into<String>) -> Self {
        self.container_identifier = Some(container_identifier.into());
        self
    }

    /// Sets the value mirroring `CKNotification.subscriptionOwnerUserRecordID`.
    pub fn with_subscription_owner_user_record_id(
        mut self,
        subscription_owner_user_record_id: CKRecordID,
    ) -> Self {
        self.subscription_owner_user_record_id = Some(subscription_owner_user_record_id);
        self
    }

    /// Sets the value mirroring `CKNotification.pruned`.
    pub fn with_pruned(mut self, is_pruned: bool) -> Self {
        self.is_pruned = is_pruned;
        self
    }

    /// Sets the value mirroring `CKNotification.subscriptionID`.
    pub fn with_subscription_id(mut self, subscription_id: impl Into<String>) -> Self {
        self.subscription_id = Some(subscription_id.into());
        self
    }
}

/// Wraps `CKQueryNotification`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKQueryNotification {
    notification: CKNotification,
    query_notification_reason: CKQueryNotificationReason,
    record_fields: BTreeMap<String, RecordValue>,
    record_id: Option<CKRecordID>,
    database_scope: CKDatabaseScope,
}

impl CKQueryNotification {
    /// Creates a wrapper mirroring `CKQueryNotification`.
    pub fn new(
        query_notification_reason: CKQueryNotificationReason,
        database_scope: CKDatabaseScope,
    ) -> Self {
        Self {
            notification: CKNotification::new(CKNotificationType::Query),
            query_notification_reason,
            record_fields: BTreeMap::new(),
            record_id: None,
            database_scope,
        }
    }

    /// Mirrors `CKQueryNotification.notification`.
    pub const fn notification(&self) -> &CKNotification {
        &self.notification
    }

    /// Mirrors `CKQueryNotification.queryNotificationReason`.
    pub const fn query_notification_reason(&self) -> CKQueryNotificationReason {
        self.query_notification_reason
    }

    /// Mirrors `CKQueryNotification.recordFields`.
    pub fn record_fields(&self) -> &BTreeMap<String, RecordValue> {
        &self.record_fields
    }

    /// Mirrors `CKQueryNotification.recordID`.
    pub const fn record_id(&self) -> Option<&CKRecordID> {
        self.record_id.as_ref()
    }

    /// Mirrors `CKQueryNotification.databaseScope`.
    pub const fn database_scope(&self) -> CKDatabaseScope {
        self.database_scope
    }

    /// Sets the value mirroring `CKQueryNotification.notification`.
    pub fn with_notification(mut self, notification: CKNotification) -> Self {
        self.notification = notification;
        self
    }

    /// Sets the value mirroring `CKQueryNotification.recordField`.
    pub fn with_record_field<V>(mut self, key: impl Into<String>, value: V) -> Self
    where
        V: Into<RecordValue>,
    {
        self.record_fields.insert(key.into(), value.into());
        self
    }

    /// Sets the value mirroring `CKQueryNotification.recordID`.
    pub fn with_record_id(mut self, record_id: CKRecordID) -> Self {
        self.record_id = Some(record_id);
        self
    }
}

/// Wraps `CKRecordZoneNotification`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKRecordZoneNotification {
    notification: CKNotification,
    record_zone_id: Option<CKRecordZoneID>,
    database_scope: CKDatabaseScope,
}

impl CKRecordZoneNotification {
    /// Creates a wrapper mirroring `CKRecordZoneNotification`.
    pub fn new(database_scope: CKDatabaseScope) -> Self {
        Self {
            notification: CKNotification::new(CKNotificationType::RecordZone),
            record_zone_id: None,
            database_scope,
        }
    }

    /// Mirrors `CKRecordZoneNotification.notification`.
    pub const fn notification(&self) -> &CKNotification {
        &self.notification
    }

    /// Mirrors `CKRecordZoneNotification.recordZoneID`.
    pub const fn record_zone_id(&self) -> Option<&CKRecordZoneID> {
        self.record_zone_id.as_ref()
    }

    /// Mirrors `CKRecordZoneNotification.databaseScope`.
    pub const fn database_scope(&self) -> CKDatabaseScope {
        self.database_scope
    }

    /// Sets the value mirroring `CKRecordZoneNotification.notification`.
    pub fn with_notification(mut self, notification: CKNotification) -> Self {
        self.notification = notification;
        self
    }

    /// Sets the value mirroring `CKRecordZoneNotification.recordZoneID`.
    pub fn with_record_zone_id(mut self, record_zone_id: CKRecordZoneID) -> Self {
        self.record_zone_id = Some(record_zone_id);
        self
    }
}

/// Wraps `CKDatabaseNotification`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKDatabaseNotification {
    notification: CKNotification,
    database_scope: CKDatabaseScope,
}

impl CKDatabaseNotification {
    /// Creates a wrapper mirroring `CKDatabaseNotification`.
    pub fn new(database_scope: CKDatabaseScope) -> Self {
        Self {
            notification: CKNotification::new(CKNotificationType::Database),
            database_scope,
        }
    }

    /// Mirrors `CKDatabaseNotification.notification`.
    pub const fn notification(&self) -> &CKNotification {
        &self.notification
    }

    /// Mirrors `CKDatabaseNotification.databaseScope`.
    pub const fn database_scope(&self) -> CKDatabaseScope {
        self.database_scope
    }

    /// Sets the value mirroring `CKDatabaseNotification.notification`.
    pub fn with_notification(mut self, notification: CKNotification) -> Self {
        self.notification = notification;
        self
    }
}
