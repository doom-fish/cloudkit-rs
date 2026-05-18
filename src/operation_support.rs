use std::sync::atomic::{AtomicU64, Ordering};

use core::ffi::c_char;
use core::ptr;

use crate::container::CKContainer;
use crate::database::CKDatabase;
use crate::error::CloudKitError;
use crate::ffi;
use crate::private::{
    cstring_from_str, error_from_status, json_cstring, opt_cstring_ptr, optional_cstring_from_str,
    parse_json_ptr, CKShareMetadataPayload,
};
use crate::record::{CKRecordZone, CKRecordZoneID};
use crate::share::{CKShare, CKShareMetadata, CKShareParticipant};
use crate::subscription::CKAnySubscription;
use crate::user_identity::CKUserIdentityLookupInfo;

static NEXT_OPERATION_IDENTIFIER: AtomicU64 = AtomicU64::new(1);

fn next_identifier(prefix: &str) -> String {
    format!(
        "{prefix}-{}",
        NEXT_OPERATION_IDENTIFIER.fetch_add(1, Ordering::Relaxed)
    )
}

/// Mirrors `CKQualityOfService`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CKQualityOfService {
    /// Mirrors `CKQualityOfService.userInteractive`.
    UserInteractive,
    /// Mirrors `CKQualityOfService.userInitiated`.
    UserInitiated,
    /// Mirrors `CKQualityOfService.default`.
    Default,
    /// Mirrors `CKQualityOfService.utility`.
    Utility,
    /// Mirrors `CKQualityOfService.background`.
    Background,
}

/// Wraps `CKOperationConfiguration`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKOperationConfiguration {
    container: Option<CKContainer>,
    quality_of_service: CKQualityOfService,
    allows_cellular_access: bool,
    long_lived: bool,
    timeout_interval_for_request: f64,
    timeout_interval_for_resource: f64,
}

impl CKOperationConfiguration {
    /// Creates a wrapper mirroring `CKOperationConfiguration`.
    pub fn new() -> Self {
        Self {
            container: None,
            quality_of_service: CKQualityOfService::Default,
            allows_cellular_access: true,
            long_lived: false,
            timeout_interval_for_request: 60.0,
            timeout_interval_for_resource: 60.0 * 60.0 * 24.0 * 7.0,
        }
    }

    /// Mirrors `CKOperationConfiguration.container`.
    pub const fn container(&self) -> Option<&CKContainer> {
        self.container.as_ref()
    }

    /// Mirrors `CKOperationConfiguration.qualityOfService`.
    pub const fn quality_of_service(&self) -> CKQualityOfService {
        self.quality_of_service
    }

    /// Mirrors `CKOperationConfiguration.allowsCellularAccess`.
    pub const fn allows_cellular_access(&self) -> bool {
        self.allows_cellular_access
    }

    /// Mirrors `CKOperationConfiguration.longLived`.
    pub const fn long_lived(&self) -> bool {
        self.long_lived
    }

    /// Mirrors `CKOperationConfiguration.timeoutIntervalForRequest`.
    pub const fn timeout_interval_for_request(&self) -> f64 {
        self.timeout_interval_for_request
    }

    /// Mirrors `CKOperationConfiguration.timeoutIntervalForResource`.
    pub const fn timeout_interval_for_resource(&self) -> f64 {
        self.timeout_interval_for_resource
    }

    /// Sets the value mirroring `CKOperationConfiguration.container`.
    pub fn with_container(mut self, container: CKContainer) -> Self {
        self.container = Some(container);
        self
    }

    /// Sets the value mirroring `CKOperationConfiguration.qualityOfService`.
    pub fn with_quality_of_service(mut self, quality_of_service: CKQualityOfService) -> Self {
        self.quality_of_service = quality_of_service;
        self
    }

    /// Sets the value mirroring `CKOperationConfiguration.allowsCellularAccess`.
    pub fn with_allows_cellular_access(mut self, allows_cellular_access: bool) -> Self {
        self.allows_cellular_access = allows_cellular_access;
        self
    }

    /// Sets the value mirroring `CKOperationConfiguration.longLived`.
    pub fn with_long_lived(mut self, long_lived: bool) -> Self {
        self.long_lived = long_lived;
        self
    }

    /// Sets the value mirroring `CKOperationConfiguration.timeoutIntervalForRequest`.
    pub fn with_timeout_interval_for_request(mut self, timeout_interval_for_request: f64) -> Self {
        self.timeout_interval_for_request = timeout_interval_for_request;
        self
    }

    /// Sets the value mirroring `CKOperationConfiguration.timeoutIntervalForResource`.
    pub fn with_timeout_interval_for_resource(
        mut self,
        timeout_interval_for_resource: f64,
    ) -> Self {
        self.timeout_interval_for_resource = timeout_interval_for_resource;
        self
    }
}

impl Default for CKOperationConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

/// Mirrors `CKOperationGroupTransferSize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CKOperationGroupTransferSize {
    /// Mirrors `CKOperationGroupTransferSize.unknown`.
    Unknown,
    /// Mirrors `CKOperationGroupTransferSize.kilobytes`.
    Kilobytes,
    /// Mirrors `CKOperationGroupTransferSize.megabytes`.
    Megabytes,
    /// Mirrors `CKOperationGroupTransferSize.tensOfMegabytes`.
    TensOfMegabytes,
    /// Mirrors `CKOperationGroupTransferSize.hundredsOfMegabytes`.
    HundredsOfMegabytes,
    /// Mirrors `CKOperationGroupTransferSize.gigabytes`.
    Gigabytes,
    /// Mirrors `CKOperationGroupTransferSize.tensOfGigabytes`.
    TensOfGigabytes,
    /// Mirrors `CKOperationGroupTransferSize.hundredsOfGigabytes`.
    HundredsOfGigabytes,
}

/// Wraps `CKOperationGroup`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKOperationGroup {
    operation_group_id: String,
    default_configuration: CKOperationConfiguration,
    name: Option<String>,
    quantity: usize,
    expected_send_size: CKOperationGroupTransferSize,
    expected_receive_size: CKOperationGroupTransferSize,
}

impl CKOperationGroup {
    /// Creates a wrapper mirroring `CKOperationGroup`.
    pub fn new() -> Self {
        Self {
            operation_group_id: next_identifier("operation-group"),
            default_configuration: CKOperationConfiguration::default(),
            name: None,
            quantity: 0,
            expected_send_size: CKOperationGroupTransferSize::Unknown,
            expected_receive_size: CKOperationGroupTransferSize::Unknown,
        }
    }

    /// Mirrors `CKOperationGroup.operationGroupID`.
    pub fn operation_group_id(&self) -> &str {
        &self.operation_group_id
    }

    /// Mirrors `CKOperationGroup.defaultConfiguration`.
    pub const fn default_configuration(&self) -> &CKOperationConfiguration {
        &self.default_configuration
    }

    /// Mirrors `CKOperationGroup.name`.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Mirrors `CKOperationGroup.quantity`.
    pub const fn quantity(&self) -> usize {
        self.quantity
    }

    /// Mirrors `CKOperationGroup.expectedSendSize`.
    pub const fn expected_send_size(&self) -> CKOperationGroupTransferSize {
        self.expected_send_size
    }

    /// Mirrors `CKOperationGroup.expectedReceiveSize`.
    pub const fn expected_receive_size(&self) -> CKOperationGroupTransferSize {
        self.expected_receive_size
    }

    /// Sets the value mirroring `CKOperationGroup.defaultConfiguration`.
    pub fn with_default_configuration(
        mut self,
        default_configuration: CKOperationConfiguration,
    ) -> Self {
        self.default_configuration = default_configuration;
        self
    }

    /// Sets the value mirroring `CKOperationGroup.name`.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the value mirroring `CKOperationGroup.quantity`.
    pub fn with_quantity(mut self, quantity: usize) -> Self {
        self.quantity = quantity;
        self
    }

    /// Sets the value mirroring `CKOperationGroup.expectedSendSize`.
    pub fn with_expected_send_size(
        mut self,
        expected_send_size: CKOperationGroupTransferSize,
    ) -> Self {
        self.expected_send_size = expected_send_size;
        self
    }

    /// Sets the value mirroring `CKOperationGroup.expectedReceiveSize`.
    pub fn with_expected_receive_size(
        mut self,
        expected_receive_size: CKOperationGroupTransferSize,
    ) -> Self {
        self.expected_receive_size = expected_receive_size;
        self
    }
}

impl Default for CKOperationGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// Wraps `CKOperation`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKOperation {
    configuration: CKOperationConfiguration,
    group: Option<CKOperationGroup>,
    operation_id: String,
    long_lived_operation_was_persisted: bool,
}

impl CKOperation {
    /// Creates a wrapper mirroring `CKOperation`.
    pub fn new() -> Self {
        Self {
            configuration: CKOperationConfiguration::default(),
            group: None,
            operation_id: next_identifier("operation"),
            long_lived_operation_was_persisted: false,
        }
    }

    /// Mirrors `CKOperation.configuration`.
    pub const fn configuration(&self) -> &CKOperationConfiguration {
        &self.configuration
    }

    /// Mirrors `CKOperation.group`.
    pub const fn group(&self) -> Option<&CKOperationGroup> {
        self.group.as_ref()
    }

    /// Mirrors `CKOperation.operationID`.
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    /// Mirrors `CKOperation.longLivedOperationWasPersisted`.
    pub const fn long_lived_operation_was_persisted(&self) -> bool {
        self.long_lived_operation_was_persisted
    }

    /// Sets the value mirroring `CKOperation.configuration`.
    pub fn with_configuration(mut self, configuration: CKOperationConfiguration) -> Self {
        self.configuration = configuration;
        self
    }

    /// Sets the value mirroring `CKOperation.group`.
    pub fn with_group(mut self, group: CKOperationGroup) -> Self {
        self.group = Some(group);
        self
    }

    /// Mirrors `CKOperation.markLongLivedOperationWasPersisted`.
    pub fn mark_long_lived_operation_was_persisted(&mut self) {
        self.long_lived_operation_was_persisted = true;
    }
}

impl Default for CKOperation {
    fn default() -> Self {
        Self::new()
    }
}

/// Wraps `CKDatabaseOperation`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKDatabaseOperation {
    operation: CKOperation,
    database: Option<CKDatabase>,
}

impl CKDatabaseOperation {
    /// Creates a wrapper mirroring `CKDatabaseOperation`.
    pub fn new() -> Self {
        Self {
            operation: CKOperation::default(),
            database: None,
        }
    }

    /// Mirrors `CKDatabaseOperation.operation`.
    pub const fn operation(&self) -> &CKOperation {
        &self.operation
    }

    /// Mirrors `CKDatabaseOperation.database`.
    pub const fn database(&self) -> Option<&CKDatabase> {
        self.database.as_ref()
    }

    /// Sets the value mirroring `CKDatabaseOperation.operation`.
    pub fn with_operation(mut self, operation: CKOperation) -> Self {
        self.operation = operation;
        self
    }

    /// Sets the value mirroring `CKDatabaseOperation.database`.
    pub fn with_database(mut self, database: CKDatabase) -> Self {
        self.operation = self.operation.clone().with_configuration(
            self.operation
                .configuration()
                .clone()
                .with_container(database.container().clone()),
        );
        self.database = Some(database);
        self
    }
}

impl Default for CKDatabaseOperation {
    fn default() -> Self {
        Self::new()
    }
}

/// Wraps `CKRecordZoneFetchResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKRecordZoneFetchResult {
    /// Mirrors `CKRecordZoneFetchResult.zoneID`.
    pub zone_id: CKRecordZoneID,
    /// Mirrors `CKRecordZoneFetchResult.recordZone`.
    pub record_zone: Option<CKRecordZone>,
    /// Mirrors `CKRecordZoneFetchResult.error`.
    pub error: Option<CloudKitError>,
}

/// Wraps `CKFetchRecordZonesResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchRecordZonesResult {
    /// Mirrors `CKFetchRecordZonesResult.recordZones`.
    pub record_zones: Vec<CKRecordZone>,
    /// Mirrors `CKFetchRecordZonesResult.results`.
    pub results: Vec<CKRecordZoneFetchResult>,
    /// Mirrors `CKFetchRecordZonesResult.operationError`.
    pub operation_error: Option<CloudKitError>,
}

/// Wraps `CKFetchRecordZonesOperation`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchRecordZonesOperation {
    database_operation: CKDatabaseOperation,
    record_zone_ids: Option<Vec<CKRecordZoneID>>,
}

impl CKFetchRecordZonesOperation {
    /// Mirrors `CKFetchRecordZonesOperation.fetchAllRecordZonesOperation`.
    pub fn fetch_all_record_zones_operation() -> Self {
        Self {
            database_operation: CKDatabaseOperation::default(),
            record_zone_ids: None,
        }
    }

    /// Creates a wrapper mirroring `CKFetchRecordZonesOperation`.
    pub fn new(record_zone_ids: Vec<CKRecordZoneID>) -> Self {
        Self {
            database_operation: CKDatabaseOperation::default(),
            record_zone_ids: Some(record_zone_ids),
        }
    }

    /// Mirrors `CKFetchRecordZonesOperation.databaseOperation`.
    pub const fn database_operation(&self) -> &CKDatabaseOperation {
        &self.database_operation
    }

    /// Mirrors `CKFetchRecordZonesOperation.recordZoneIDs`.
    pub fn record_zone_ids(&self) -> Option<&[CKRecordZoneID]> {
        self.record_zone_ids.as_deref()
    }

    /// Sets the value mirroring `CKFetchRecordZonesOperation.databaseOperation`.
    pub fn with_database_operation(mut self, database_operation: CKDatabaseOperation) -> Self {
        self.database_operation = database_operation;
        self
    }

    /// Executes the corresponding `CloudKit` operation in `CKDatabase`.
    pub fn execute_in(
        &self,
        database: &CKDatabase,
    ) -> Result<CKFetchRecordZonesResult, CloudKitError> {
        let mut record_zones = Vec::new();
        let mut results = Vec::new();
        if let Some(zone_ids) = &self.record_zone_ids {
            for zone_id in zone_ids {
                match database.fetch_record_zone(zone_id) {
                    Ok(record_zone) => {
                        record_zones.push(record_zone.clone());
                        results.push(CKRecordZoneFetchResult {
                            zone_id: zone_id.clone(),
                            record_zone: Some(record_zone),
                            error: None,
                        });
                    }
                    Err(error) => results.push(CKRecordZoneFetchResult {
                        zone_id: zone_id.clone(),
                        record_zone: None,
                        error: Some(error),
                    }),
                }
            }
        } else {
            for record_zone in database.fetch_all_record_zones()? {
                results.push(CKRecordZoneFetchResult {
                    zone_id: record_zone.zone_id().clone(),
                    record_zone: Some(record_zone.clone()),
                    error: None,
                });
                record_zones.push(record_zone);
            }
        }
        Ok(CKFetchRecordZonesResult {
            record_zones,
            results,
            operation_error: None,
        })
    }
}

/// Wraps `CKRecordZoneSaveResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKRecordZoneSaveResult {
    /// Mirrors `CKRecordZoneSaveResult.zoneID`.
    pub zone_id: CKRecordZoneID,
    /// Mirrors `CKRecordZoneSaveResult.recordZone`.
    pub record_zone: Option<CKRecordZone>,
    /// Mirrors `CKRecordZoneSaveResult.error`.
    pub error: Option<CloudKitError>,
}

/// Wraps `CKRecordZoneDeleteResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKRecordZoneDeleteResult {
    /// Mirrors `CKRecordZoneDeleteResult.zoneID`.
    pub zone_id: CKRecordZoneID,
    /// Mirrors `CKRecordZoneDeleteResult.error`.
    pub error: Option<CloudKitError>,
}

/// Wraps `ModifyRecordZonesResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct ModifyRecordZonesResult {
    /// Mirrors `ModifyRecordZonesResult.savedRecordZones`.
    pub saved_record_zones: Vec<CKRecordZone>,
    /// Mirrors `ModifyRecordZonesResult.deletedRecordZoneIDs`.
    pub deleted_record_zone_ids: Vec<CKRecordZoneID>,
    /// Mirrors `ModifyRecordZonesResult.saveResults`.
    pub save_results: Vec<CKRecordZoneSaveResult>,
    /// Mirrors `ModifyRecordZonesResult.deleteResults`.
    pub delete_results: Vec<CKRecordZoneDeleteResult>,
    /// Mirrors `ModifyRecordZonesResult.operationError`.
    pub operation_error: Option<CloudKitError>,
}

/// Wraps `CKModifyRecordZonesOperation`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKModifyRecordZonesOperation {
    database_operation: CKDatabaseOperation,
    record_zones_to_save: Vec<CKRecordZone>,
    record_zone_ids_to_delete: Vec<CKRecordZoneID>,
}

impl CKModifyRecordZonesOperation {
    /// Creates a wrapper mirroring `CKModifyRecordZonesOperation`.
    pub fn new(
        record_zones_to_save: Vec<CKRecordZone>,
        record_zone_ids_to_delete: Vec<CKRecordZoneID>,
    ) -> Self {
        Self {
            database_operation: CKDatabaseOperation::default(),
            record_zones_to_save,
            record_zone_ids_to_delete,
        }
    }

    /// Mirrors `CKModifyRecordZonesOperation.databaseOperation`.
    pub const fn database_operation(&self) -> &CKDatabaseOperation {
        &self.database_operation
    }

    /// Mirrors `CKModifyRecordZonesOperation.recordZonesToSave`.
    pub fn record_zones_to_save(&self) -> &[CKRecordZone] {
        &self.record_zones_to_save
    }

    /// Mirrors `CKModifyRecordZonesOperation.recordZoneIDsToDelete`.
    pub fn record_zone_ids_to_delete(&self) -> &[CKRecordZoneID] {
        &self.record_zone_ids_to_delete
    }

    /// Sets the value mirroring `CKModifyRecordZonesOperation.databaseOperation`.
    pub fn with_database_operation(mut self, database_operation: CKDatabaseOperation) -> Self {
        self.database_operation = database_operation;
        self
    }

    /// Executes the corresponding `CloudKit` operation in `CKDatabase`.
    pub fn execute_in(
        &self,
        database: &CKDatabase,
    ) -> Result<ModifyRecordZonesResult, CloudKitError> {
        let mut saved_record_zones = Vec::new();
        let mut deleted_record_zone_ids = Vec::new();
        let mut save_results = Vec::new();
        let mut delete_results = Vec::new();

        for record_zone in &self.record_zones_to_save {
            match database.save_record_zone(record_zone) {
                Ok(saved_zone) => {
                    saved_record_zones.push(saved_zone.clone());
                    save_results.push(CKRecordZoneSaveResult {
                        zone_id: saved_zone.zone_id().clone(),
                        record_zone: Some(saved_zone),
                        error: None,
                    });
                }
                Err(error) => save_results.push(CKRecordZoneSaveResult {
                    zone_id: record_zone.zone_id().clone(),
                    record_zone: None,
                    error: Some(error),
                }),
            }
        }

        for zone_id in &self.record_zone_ids_to_delete {
            match database.delete_record_zone(zone_id) {
                Ok(deleted_zone_id) => {
                    deleted_record_zone_ids.push(deleted_zone_id.clone());
                    delete_results.push(CKRecordZoneDeleteResult {
                        zone_id: deleted_zone_id,
                        error: None,
                    });
                }
                Err(error) => delete_results.push(CKRecordZoneDeleteResult {
                    zone_id: zone_id.clone(),
                    error: Some(error),
                }),
            }
        }

        Ok(ModifyRecordZonesResult {
            saved_record_zones,
            deleted_record_zone_ids,
            save_results,
            delete_results,
            operation_error: None,
        })
    }
}

/// Wraps `CKSubscriptionFetchResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKSubscriptionFetchResult {
    /// Mirrors `CKSubscriptionFetchResult.subscriptionID`.
    pub subscription_id: String,
    /// Mirrors `CKSubscriptionFetchResult.subscription`.
    pub subscription: Option<CKAnySubscription>,
    /// Mirrors `CKSubscriptionFetchResult.error`.
    pub error: Option<CloudKitError>,
}

/// Wraps `CKFetchSubscriptionsResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchSubscriptionsResult {
    /// Mirrors `CKFetchSubscriptionsResult.subscriptions`.
    pub subscriptions: Vec<CKAnySubscription>,
    /// Mirrors `CKFetchSubscriptionsResult.results`.
    pub results: Vec<CKSubscriptionFetchResult>,
    /// Mirrors `CKFetchSubscriptionsResult.operationError`.
    pub operation_error: Option<CloudKitError>,
}

/// Wraps `CKFetchSubscriptionsOperation`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchSubscriptionsOperation {
    database_operation: CKDatabaseOperation,
    subscription_ids: Option<Vec<String>>,
}

impl CKFetchSubscriptionsOperation {
    /// Mirrors `CKFetchSubscriptionsOperation.fetchAllSubscriptionsOperation`.
    pub fn fetch_all_subscriptions_operation() -> Self {
        Self {
            database_operation: CKDatabaseOperation::default(),
            subscription_ids: None,
        }
    }

    /// Creates a wrapper mirroring `CKFetchSubscriptionsOperation`.
    pub fn new(subscription_ids: Vec<String>) -> Self {
        Self {
            database_operation: CKDatabaseOperation::default(),
            subscription_ids: Some(subscription_ids),
        }
    }

    /// Mirrors `CKFetchSubscriptionsOperation.databaseOperation`.
    pub const fn database_operation(&self) -> &CKDatabaseOperation {
        &self.database_operation
    }

    /// Mirrors `CKFetchSubscriptionsOperation.subscriptionIDs`.
    pub fn subscription_ids(&self) -> Option<&[String]> {
        self.subscription_ids.as_deref()
    }

    /// Sets the value mirroring `CKFetchSubscriptionsOperation.databaseOperation`.
    pub fn with_database_operation(mut self, database_operation: CKDatabaseOperation) -> Self {
        self.database_operation = database_operation;
        self
    }

    /// Executes the corresponding `CloudKit` operation in `CKDatabase`.
    pub fn execute_in(
        &self,
        database: &CKDatabase,
    ) -> Result<CKFetchSubscriptionsResult, CloudKitError> {
        let mut subscriptions = Vec::new();
        let mut results = Vec::new();
        if let Some(subscription_ids) = &self.subscription_ids {
            for subscription_id in subscription_ids {
                match database.fetch_subscription(subscription_id) {
                    Ok(subscription) => {
                        subscriptions.push(subscription.clone());
                        results.push(CKSubscriptionFetchResult {
                            subscription_id: subscription_id.clone(),
                            subscription: Some(subscription),
                            error: None,
                        });
                    }
                    Err(error) => results.push(CKSubscriptionFetchResult {
                        subscription_id: subscription_id.clone(),
                        subscription: None,
                        error: Some(error),
                    }),
                }
            }
        } else {
            for subscription in database.fetch_all_subscriptions()? {
                let subscription_id = subscription.subscription_id().to_string();
                results.push(CKSubscriptionFetchResult {
                    subscription_id,
                    subscription: Some(subscription.clone()),
                    error: None,
                });
                subscriptions.push(subscription);
            }
        }
        Ok(CKFetchSubscriptionsResult {
            subscriptions,
            results,
            operation_error: None,
        })
    }
}

/// Wraps `CKSubscriptionSaveResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKSubscriptionSaveResult {
    /// Mirrors `CKSubscriptionSaveResult.subscriptionID`.
    pub subscription_id: String,
    /// Mirrors `CKSubscriptionSaveResult.subscription`.
    pub subscription: Option<CKAnySubscription>,
    /// Mirrors `CKSubscriptionSaveResult.error`.
    pub error: Option<CloudKitError>,
}

/// Wraps `CKSubscriptionDeleteResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKSubscriptionDeleteResult {
    /// Mirrors `CKSubscriptionDeleteResult.subscriptionID`.
    pub subscription_id: String,
    /// Mirrors `CKSubscriptionDeleteResult.error`.
    pub error: Option<CloudKitError>,
}

/// Wraps `ModifySubscriptionsResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct ModifySubscriptionsResult {
    /// Mirrors `ModifySubscriptionsResult.savedSubscriptions`.
    pub saved_subscriptions: Vec<CKAnySubscription>,
    /// Mirrors `ModifySubscriptionsResult.deletedSubscriptionIDs`.
    pub deleted_subscription_ids: Vec<String>,
    /// Mirrors `ModifySubscriptionsResult.saveResults`.
    pub save_results: Vec<CKSubscriptionSaveResult>,
    /// Mirrors `ModifySubscriptionsResult.deleteResults`.
    pub delete_results: Vec<CKSubscriptionDeleteResult>,
    /// Mirrors `ModifySubscriptionsResult.operationError`.
    pub operation_error: Option<CloudKitError>,
}

/// Wraps `CKModifySubscriptionsOperation`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKModifySubscriptionsOperation {
    database_operation: CKDatabaseOperation,
    subscriptions_to_save: Vec<CKAnySubscription>,
    subscription_ids_to_delete: Vec<String>,
}

impl CKModifySubscriptionsOperation {
    /// Creates a wrapper mirroring `CKModifySubscriptionsOperation`.
    pub fn new(
        subscriptions_to_save: Vec<CKAnySubscription>,
        subscription_ids_to_delete: Vec<String>,
    ) -> Self {
        Self {
            database_operation: CKDatabaseOperation::default(),
            subscriptions_to_save,
            subscription_ids_to_delete,
        }
    }

    /// Mirrors `CKModifySubscriptionsOperation.databaseOperation`.
    pub const fn database_operation(&self) -> &CKDatabaseOperation {
        &self.database_operation
    }

    /// Mirrors `CKModifySubscriptionsOperation.subscriptionsToSave`.
    pub fn subscriptions_to_save(&self) -> &[CKAnySubscription] {
        &self.subscriptions_to_save
    }

    /// Mirrors `CKModifySubscriptionsOperation.subscriptionIDsToDelete`.
    pub fn subscription_ids_to_delete(&self) -> &[String] {
        &self.subscription_ids_to_delete
    }

    /// Sets the value mirroring `CKModifySubscriptionsOperation.databaseOperation`.
    pub fn with_database_operation(mut self, database_operation: CKDatabaseOperation) -> Self {
        self.database_operation = database_operation;
        self
    }

    /// Executes the corresponding `CloudKit` operation in `CKDatabase`.
    pub fn execute_in(
        &self,
        database: &CKDatabase,
    ) -> Result<ModifySubscriptionsResult, CloudKitError> {
        let mut saved_subscriptions = Vec::new();
        let mut deleted_subscription_ids = Vec::new();
        let mut save_results = Vec::new();
        let mut delete_results = Vec::new();

        for subscription in &self.subscriptions_to_save {
            match database.save_subscription(subscription.clone()) {
                Ok(saved_subscription) => {
                    let subscription_id = saved_subscription.subscription_id().to_string();
                    saved_subscriptions.push(saved_subscription.clone());
                    save_results.push(CKSubscriptionSaveResult {
                        subscription_id,
                        subscription: Some(saved_subscription),
                        error: None,
                    });
                }
                Err(error) => save_results.push(CKSubscriptionSaveResult {
                    subscription_id: subscription.subscription_id().to_string(),
                    subscription: None,
                    error: Some(error),
                }),
            }
        }

        for subscription_id in &self.subscription_ids_to_delete {
            match database.delete_subscription(subscription_id) {
                Ok(deleted_subscription_id) => {
                    deleted_subscription_ids.push(deleted_subscription_id.clone());
                    delete_results.push(CKSubscriptionDeleteResult {
                        subscription_id: deleted_subscription_id,
                        error: None,
                    });
                }
                Err(error) => delete_results.push(CKSubscriptionDeleteResult {
                    subscription_id: subscription_id.clone(),
                    error: Some(error),
                }),
            }
        }

        Ok(ModifySubscriptionsResult {
            saved_subscriptions,
            deleted_subscription_ids,
            save_results,
            delete_results,
            operation_error: None,
        })
    }
}

/// Wraps `CKFetchWebAuthTokenOperation`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchWebAuthTokenOperation {
    database_operation: CKDatabaseOperation,
    api_token: Option<String>,
}

impl CKFetchWebAuthTokenOperation {
    /// Creates a wrapper mirroring `CKFetchWebAuthTokenOperation`.
    pub fn new() -> Self {
        Self {
            database_operation: CKDatabaseOperation::default(),
            api_token: None,
        }
    }

    /// Mirrors `CKFetchWebAuthTokenOperation.databaseOperation`.
    pub const fn database_operation(&self) -> &CKDatabaseOperation {
        &self.database_operation
    }

    /// Mirrors `CKFetchWebAuthTokenOperation.apiToken`.
    pub fn api_token(&self) -> Option<&str> {
        self.api_token.as_deref()
    }

    /// Sets the value mirroring `CKFetchWebAuthTokenOperation.databaseOperation`.
    pub fn with_database_operation(mut self, database_operation: CKDatabaseOperation) -> Self {
        self.database_operation = database_operation;
        self
    }

    /// Sets the value mirroring `CKFetchWebAuthTokenOperation.apiToken`.
    pub fn with_api_token(mut self, api_token: impl Into<String>) -> Self {
        self.api_token = Some(api_token.into());
        self
    }

    /// Executes the corresponding `CloudKit` operation in `CKDatabase`.
    pub fn execute_in(&self, database: &CKDatabase) -> Result<String, CloudKitError> {
        let identifier = optional_cstring_from_str(
            database.container().container_identifier(),
            "container identifier",
        )?;
        let api_token = cstring_from_str(
            self.api_token.as_deref().ok_or_else(|| {
                CloudKitError::bridge(-1, "CKFetchWebAuthTokenOperation requires an API token")
            })?,
            "API token",
        )?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        // SAFETY: all pointer arguments are either null (via `opt_cstring_ptr`) or valid null-terminated C strings whose lifetimes cover the call. Any output pointers are valid mutable pointers; owned string outputs such as `out_json` and `out_error` start as null so the bridge can allocate and transfer ownership.
        let status = unsafe {
            ffi::ck_database_execute_fetch_web_auth_token_sync(
                opt_cstring_ptr(&identifier),
                database.database_scope() as i32,
                api_token.as_ptr(),
                &mut out_json,
                &mut out_error,
            )
        };
        if status != ffi::status::OK {
            // SAFETY: `out_error` is either null or a bridge-allocated C string; `error_from_status` consumes it.
            return Err(unsafe { error_from_status(status, out_error) });
        }
        // SAFETY: `out_json` is either null or a bridge-allocated C string; `parse_json_ptr` frees it via `ck_string_free`.
        unsafe { parse_json_ptr::<String>(out_json, "web auth token") }
    }
}

impl Default for CKFetchWebAuthTokenOperation {
    fn default() -> Self {
        Self::new()
    }
}

/// Wraps `CKShareParticipantFetchResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKShareParticipantFetchResult {
    /// Mirrors `CKShareParticipantFetchResult.lookupInfo`.
    pub lookup_info: CKUserIdentityLookupInfo,
    /// Mirrors `CKShareParticipantFetchResult.participant`.
    pub participant: Option<CKShareParticipant>,
    /// Mirrors `CKShareParticipantFetchResult.error`.
    pub error: Option<CloudKitError>,
}

/// Wraps `CKFetchShareParticipantsResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchShareParticipantsResult {
    /// Mirrors `CKFetchShareParticipantsResult.participants`.
    pub participants: Vec<CKShareParticipant>,
    /// Mirrors `CKFetchShareParticipantsResult.results`.
    pub results: Vec<CKShareParticipantFetchResult>,
    /// Mirrors `CKFetchShareParticipantsResult.operationError`.
    pub operation_error: Option<CloudKitError>,
}

/// Wraps `CKFetchShareParticipantsOperation`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchShareParticipantsOperation {
    operation: CKOperation,
    user_identity_lookup_infos: Vec<CKUserIdentityLookupInfo>,
}

impl CKFetchShareParticipantsOperation {
    /// Creates a wrapper mirroring `CKFetchShareParticipantsOperation`.
    pub fn new() -> Self {
        Self {
            operation: CKOperation::default(),
            user_identity_lookup_infos: Vec::new(),
        }
    }

    /// Mirrors `CKFetchShareParticipantsOperation.operation`.
    pub const fn operation(&self) -> &CKOperation {
        &self.operation
    }

    /// Mirrors `CKFetchShareParticipantsOperation.userIDentityLookupInfos`.
    pub fn user_identity_lookup_infos(&self) -> &[CKUserIdentityLookupInfo] {
        &self.user_identity_lookup_infos
    }

    /// Sets the value mirroring `CKFetchShareParticipantsOperation.operation`.
    pub fn with_operation(mut self, operation: CKOperation) -> Self {
        self.operation = operation;
        self
    }

    /// Sets the value mirroring `CKFetchShareParticipantsOperation.userIDentityLookupInfos`.
    pub fn with_user_identity_lookup_infos(
        mut self,
        user_identity_lookup_infos: Vec<CKUserIdentityLookupInfo>,
    ) -> Self {
        self.user_identity_lookup_infos = user_identity_lookup_infos;
        self
    }

    /// Executes the corresponding `CloudKit` operation in `CKDatabase`.
    pub fn execute_in(
        &self,
        container: &CKContainer,
    ) -> Result<CKFetchShareParticipantsResult, CloudKitError> {
        let mut participants = Vec::new();
        let mut results = Vec::new();
        for lookup_info in &self.user_identity_lookup_infos {
            match container.fetch_share_participant(lookup_info) {
                Ok(participant) => {
                    participants.push(participant.clone());
                    results.push(CKShareParticipantFetchResult {
                        lookup_info: lookup_info.clone(),
                        participant: Some(participant),
                        error: None,
                    });
                }
                Err(error) => results.push(CKShareParticipantFetchResult {
                    lookup_info: lookup_info.clone(),
                    participant: None,
                    error: Some(error),
                }),
            }
        }
        Ok(CKFetchShareParticipantsResult {
            participants,
            results,
            operation_error: None,
        })
    }
}

impl Default for CKFetchShareParticipantsOperation {
    fn default() -> Self {
        Self::new()
    }
}

/// Wraps `CKShareMetadataFetchResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKShareMetadataFetchResult {
    /// Mirrors `CKShareMetadataFetchResult.shareURL`.
    pub share_url: String,
    /// Mirrors `CKShareMetadataFetchResult.shareMetadata`.
    pub share_metadata: Option<CKShareMetadata>,
    /// Mirrors `CKShareMetadataFetchResult.error`.
    pub error: Option<CloudKitError>,
}

/// Wraps `CKFetchShareMetadataResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchShareMetadataResult {
    /// Mirrors `CKFetchShareMetadataResult.shareMetadatas`.
    pub share_metadatas: Vec<CKShareMetadata>,
    /// Mirrors `CKFetchShareMetadataResult.results`.
    pub results: Vec<CKShareMetadataFetchResult>,
    /// Mirrors `CKFetchShareMetadataResult.operationError`.
    pub operation_error: Option<CloudKitError>,
}

/// Wraps `CKFetchShareMetadataOperation`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchShareMetadataOperation {
    operation: CKOperation,
    share_urls: Vec<String>,
    should_fetch_root_record: bool,
    root_record_desired_keys: Option<Vec<String>>,
}

impl CKFetchShareMetadataOperation {
    /// Creates a wrapper mirroring `CKFetchShareMetadataOperation`.
    pub fn new() -> Self {
        Self {
            operation: CKOperation::default(),
            share_urls: Vec::new(),
            should_fetch_root_record: false,
            root_record_desired_keys: None,
        }
    }

    /// Mirrors `CKFetchShareMetadataOperation.operation`.
    pub const fn operation(&self) -> &CKOperation {
        &self.operation
    }

    /// Mirrors `CKFetchShareMetadataOperation.shareURLs`.
    pub fn share_urls(&self) -> &[String] {
        &self.share_urls
    }

    /// Mirrors `CKFetchShareMetadataOperation.shouldFetchRootRecord`.
    pub const fn should_fetch_root_record(&self) -> bool {
        self.should_fetch_root_record
    }

    /// Mirrors `CKFetchShareMetadataOperation.rootRecordDesiredKeys`.
    pub fn root_record_desired_keys(&self) -> Option<&[String]> {
        self.root_record_desired_keys.as_deref()
    }

    /// Sets the value mirroring `CKFetchShareMetadataOperation.operation`.
    pub fn with_operation(mut self, operation: CKOperation) -> Self {
        self.operation = operation;
        self
    }

    /// Sets the value mirroring `CKFetchShareMetadataOperation.shareURLs`.
    pub fn with_share_urls(mut self, share_urls: Vec<String>) -> Self {
        self.share_urls = share_urls;
        self
    }

    /// Sets the value mirroring `CKFetchShareMetadataOperation.shouldFetchRootRecord`.
    pub fn with_should_fetch_root_record(mut self, should_fetch_root_record: bool) -> Self {
        self.should_fetch_root_record = should_fetch_root_record;
        self
    }

    /// Sets the value mirroring `CKFetchShareMetadataOperation.rootRecordDesiredKeys`.
    pub fn with_root_record_desired_keys(mut self, root_record_desired_keys: Vec<String>) -> Self {
        self.root_record_desired_keys = Some(root_record_desired_keys);
        self
    }

    /// Executes the corresponding `CloudKit` operation in `CKDatabase`.
    pub fn execute_in(
        &self,
        container: &CKContainer,
    ) -> Result<CKFetchShareMetadataResult, CloudKitError> {
        let identifier =
            optional_cstring_from_str(container.container_identifier(), "container identifier")?;
        let desired_keys_json = self
            .root_record_desired_keys
            .as_ref()
            .map(|keys| json_cstring(keys, "root record desired keys"))
            .transpose()?;
        let mut share_metadatas = Vec::new();
        let mut results = Vec::new();

        for share_url in &self.share_urls {
            let share_url = cstring_from_str(share_url, "share URL")?;
            let mut out_json: *mut c_char = ptr::null_mut();
            let mut out_error: *mut c_char = ptr::null_mut();
            // SAFETY: all pointer arguments are either null (via `opt_cstring_ptr`) or valid null-terminated C strings whose lifetimes cover the call. Any output pointers are valid mutable pointers; owned string outputs such as `out_json` and `out_error` start as null so the bridge can allocate and transfer ownership.
            let status = unsafe {
                ffi::ck_container_fetch_share_metadata_sync(
                    opt_cstring_ptr(&identifier),
                    share_url.as_ptr(),
                    self.should_fetch_root_record,
                    opt_cstring_ptr(&desired_keys_json),
                    &mut out_json,
                    &mut out_error,
                )
            };
            if status == ffi::status::OK {
                // SAFETY: `out_json` is either null or a bridge-allocated C string; `parse_json_ptr` frees it via `ck_string_free`.
                let payload = unsafe {
                    parse_json_ptr::<CKShareMetadataPayload>(out_json, "share metadata")?
                };
                let share_metadata = CKShareMetadata::from_payload(payload);
                share_metadatas.push(share_metadata.clone());
                results.push(CKShareMetadataFetchResult {
                    share_url: share_url.as_c_str().to_string_lossy().into_owned(),
                    share_metadata: Some(share_metadata),
                    error: None,
                });
            } else {
                results.push(CKShareMetadataFetchResult {
                    share_url: share_url.as_c_str().to_string_lossy().into_owned(),
                    share_metadata: None,
                    // SAFETY: `out_error` is either null or a bridge-allocated C string; `error_from_status` consumes it.
                    error: Some(unsafe { error_from_status(status, out_error) }),
                });
            }
        }

        Ok(CKFetchShareMetadataResult {
            share_metadatas,
            results,
            operation_error: None,
        })
    }
}

impl Default for CKFetchShareMetadataOperation {
    fn default() -> Self {
        Self::new()
    }
}

/// Wraps `CKAcceptShareResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKAcceptShareResult {
    /// Mirrors `CKAcceptShareResult.shareMetadata`.
    pub share_metadata: CKShareMetadata,
    /// Mirrors `CKAcceptShareResult.acceptedShare`.
    pub accepted_share: Option<CKShare>,
    /// Mirrors `CKAcceptShareResult.error`.
    pub error: Option<CloudKitError>,
}

/// Wraps `CKAcceptSharesResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKAcceptSharesResult {
    /// Mirrors `CKAcceptSharesResult.acceptedShares`.
    pub accepted_shares: Vec<CKShare>,
    /// Mirrors `CKAcceptSharesResult.results`.
    pub results: Vec<CKAcceptShareResult>,
    /// Mirrors `CKAcceptSharesResult.operationError`.
    pub operation_error: Option<CloudKitError>,
}

/// Wraps `CKAcceptSharesOperation`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKAcceptSharesOperation {
    operation: CKOperation,
    share_metadatas: Vec<CKShareMetadata>,
}

impl CKAcceptSharesOperation {
    /// Creates a wrapper mirroring `CKAcceptSharesOperation`.
    pub fn new() -> Self {
        Self {
            operation: CKOperation::default(),
            share_metadatas: Vec::new(),
        }
    }

    /// Mirrors `CKAcceptSharesOperation.operation`.
    pub const fn operation(&self) -> &CKOperation {
        &self.operation
    }

    /// Mirrors `CKAcceptSharesOperation.shareMetadatas`.
    pub fn share_metadatas(&self) -> &[CKShareMetadata] {
        &self.share_metadatas
    }

    /// Sets the value mirroring `CKAcceptSharesOperation.operation`.
    pub fn with_operation(mut self, operation: CKOperation) -> Self {
        self.operation = operation;
        self
    }

    /// Sets the value mirroring `CKAcceptSharesOperation.shareMetadatas`.
    pub fn with_share_metadatas(mut self, share_metadatas: Vec<CKShareMetadata>) -> Self {
        self.share_metadatas = share_metadatas;
        self
    }

    /// Executes the corresponding `CloudKit` operation in `CKDatabase`.
    pub fn execute_in(
        &self,
        container: &CKContainer,
    ) -> Result<CKAcceptSharesResult, CloudKitError> {
        let identifier =
            optional_cstring_from_str(container.container_identifier(), "container identifier")?;
        let mut accepted_shares = Vec::new();
        let mut results = Vec::new();
        for share_metadata in &self.share_metadatas {
            let share_metadata_json = json_cstring(&share_metadata.to_payload(), "share metadata")?;
            let mut out_json: *mut c_char = ptr::null_mut();
            let mut out_error: *mut c_char = ptr::null_mut();
            // SAFETY: all pointer arguments are either null (via `opt_cstring_ptr`) or valid null-terminated C strings whose lifetimes cover the call. Any output pointers are valid mutable pointers; owned string outputs such as `out_json` and `out_error` start as null so the bridge can allocate and transfer ownership.
            let status = unsafe {
                ffi::ck_container_accept_share_metadata_sync(
                    opt_cstring_ptr(&identifier),
                    share_metadata_json.as_ptr(),
                    &mut out_json,
                    &mut out_error,
                )
            };
            if status == ffi::status::OK {
                // SAFETY: `out_json` is either null or a bridge-allocated C string; `parse_json_ptr` frees it via `ck_string_free`.
                let payload = unsafe { parse_json_ptr(out_json, "accepted share")? };
                let accepted_share = CKShare::from_payload(payload);
                accepted_shares.push(accepted_share.clone());
                results.push(CKAcceptShareResult {
                    share_metadata: share_metadata.clone(),
                    accepted_share: Some(accepted_share),
                    error: None,
                });
            } else {
                results.push(CKAcceptShareResult {
                    share_metadata: share_metadata.clone(),
                    accepted_share: None,
                    // SAFETY: `out_error` is either null or a bridge-allocated C string; `error_from_status` consumes it.
                    error: Some(unsafe { error_from_status(status, out_error) }),
                });
            }
        }
        Ok(CKAcceptSharesResult {
            accepted_shares,
            results,
            operation_error: None,
        })
    }
}

impl Default for CKAcceptSharesOperation {
    fn default() -> Self {
        Self::new()
    }
}

/// Wraps `CKShareAccessRequestResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKShareAccessRequestResult {
    /// Mirrors `CKShareAccessRequestResult.shareURL`.
    pub share_url: String,
    /// Mirrors `CKShareAccessRequestResult.error`.
    pub error: Option<CloudKitError>,
}

/// Wraps `CKShareRequestAccessResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKShareRequestAccessResult {
    /// Mirrors `CKShareRequestAccessResult.results`.
    pub results: Vec<CKShareAccessRequestResult>,
    /// Mirrors `CKShareRequestAccessResult.operationError`.
    pub operation_error: Option<CloudKitError>,
}

/// Wraps `CKShareRequestAccessOperation`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKShareRequestAccessOperation {
    operation: CKOperation,
    share_urls: Vec<String>,
}

impl CKShareRequestAccessOperation {
    /// Creates a wrapper mirroring `CKShareRequestAccessOperation`.
    pub fn new() -> Self {
        Self {
            operation: CKOperation::default(),
            share_urls: Vec::new(),
        }
    }

    /// Mirrors `CKShareRequestAccessOperation.operation`.
    pub const fn operation(&self) -> &CKOperation {
        &self.operation
    }

    /// Mirrors `CKShareRequestAccessOperation.shareURLs`.
    pub fn share_urls(&self) -> &[String] {
        &self.share_urls
    }

    /// Sets the value mirroring `CKShareRequestAccessOperation.operation`.
    pub fn with_operation(mut self, operation: CKOperation) -> Self {
        self.operation = operation;
        self
    }

    /// Sets the value mirroring `CKShareRequestAccessOperation.shareURLs`.
    pub fn with_share_urls(mut self, share_urls: Vec<String>) -> Self {
        self.share_urls = share_urls;
        self
    }

    /// Executes the corresponding `CloudKit` operation in `CKDatabase`.
    pub fn execute_in(
        &self,
        container: &CKContainer,
    ) -> Result<CKShareRequestAccessResult, CloudKitError> {
        let identifier =
            optional_cstring_from_str(container.container_identifier(), "container identifier")?;
        let mut results = Vec::new();
        for share_url in &self.share_urls {
            let share_url = cstring_from_str(share_url, "share URL")?;
            let mut out_error: *mut c_char = ptr::null_mut();
            // SAFETY: all pointer arguments are either null (via `opt_cstring_ptr`) or valid null-terminated C strings whose lifetimes cover the call. Any output pointers are valid mutable pointers; owned string outputs such as `out_json` and `out_error` start as null so the bridge can allocate and transfer ownership.
            let status = unsafe {
                ffi::ck_container_request_share_access_sync(
                    opt_cstring_ptr(&identifier),
                    share_url.as_ptr(),
                    &mut out_error,
                )
            };
            results.push(CKShareAccessRequestResult {
                share_url: share_url.as_c_str().to_string_lossy().into_owned(),
                error: if status == ffi::status::OK {
                    None
                } else {
                    // SAFETY: `out_error` is either null or a bridge-allocated C string; `error_from_status` consumes it.
                    Some(unsafe { error_from_status(status, out_error) })
                },
            });
        }
        Ok(CKShareRequestAccessResult {
            results,
            operation_error: None,
        })
    }
}

impl Default for CKShareRequestAccessOperation {
    fn default() -> Self {
        Self::new()
    }
}
