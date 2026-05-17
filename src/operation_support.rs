use std::sync::atomic::{AtomicU64, Ordering};

use core::ffi::c_char;
use core::ptr;

use crate::container::CKContainer;
use crate::database::CKDatabase;
use crate::error::CloudKitError;
use crate::ffi;
use crate::private::{
    cstring_from_str, error_from_status, json_cstring, opt_cstring_ptr,
    optional_cstring_from_str, parse_json_ptr, CKShareMetadataPayload,
};
use crate::record::{CKRecordZone, CKRecordZoneID};
use crate::share::{CKShare, CKShareMetadata, CKShareParticipant};
use crate::subscription::CKAnySubscription;
use crate::user_identity::CKUserIdentityLookupInfo;

static NEXT_OPERATION_IDENTIFIER: AtomicU64 = AtomicU64::new(1);

fn next_identifier(prefix: &str) -> String {
    format!("{prefix}-{}", NEXT_OPERATION_IDENTIFIER.fetch_add(1, Ordering::Relaxed))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CKQualityOfService {
    UserInteractive,
    UserInitiated,
    Default,
    Utility,
    Background,
}

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

    pub const fn container(&self) -> Option<&CKContainer> {
        self.container.as_ref()
    }

    pub const fn quality_of_service(&self) -> CKQualityOfService {
        self.quality_of_service
    }

    pub const fn allows_cellular_access(&self) -> bool {
        self.allows_cellular_access
    }

    pub const fn long_lived(&self) -> bool {
        self.long_lived
    }

    pub const fn timeout_interval_for_request(&self) -> f64 {
        self.timeout_interval_for_request
    }

    pub const fn timeout_interval_for_resource(&self) -> f64 {
        self.timeout_interval_for_resource
    }

    pub fn with_container(mut self, container: CKContainer) -> Self {
        self.container = Some(container);
        self
    }

    pub fn with_quality_of_service(mut self, quality_of_service: CKQualityOfService) -> Self {
        self.quality_of_service = quality_of_service;
        self
    }

    pub fn with_allows_cellular_access(mut self, allows_cellular_access: bool) -> Self {
        self.allows_cellular_access = allows_cellular_access;
        self
    }

    pub fn with_long_lived(mut self, long_lived: bool) -> Self {
        self.long_lived = long_lived;
        self
    }

    pub fn with_timeout_interval_for_request(mut self, timeout_interval_for_request: f64) -> Self {
        self.timeout_interval_for_request = timeout_interval_for_request;
        self
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CKOperationGroupTransferSize {
    Unknown,
    Kilobytes,
    Megabytes,
    TensOfMegabytes,
    HundredsOfMegabytes,
    Gigabytes,
    TensOfGigabytes,
    HundredsOfGigabytes,
}

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

    pub fn operation_group_id(&self) -> &str {
        &self.operation_group_id
    }

    pub const fn default_configuration(&self) -> &CKOperationConfiguration {
        &self.default_configuration
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub const fn quantity(&self) -> usize {
        self.quantity
    }

    pub const fn expected_send_size(&self) -> CKOperationGroupTransferSize {
        self.expected_send_size
    }

    pub const fn expected_receive_size(&self) -> CKOperationGroupTransferSize {
        self.expected_receive_size
    }

    pub fn with_default_configuration(
        mut self,
        default_configuration: CKOperationConfiguration,
    ) -> Self {
        self.default_configuration = default_configuration;
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_quantity(mut self, quantity: usize) -> Self {
        self.quantity = quantity;
        self
    }

    pub fn with_expected_send_size(
        mut self,
        expected_send_size: CKOperationGroupTransferSize,
    ) -> Self {
        self.expected_send_size = expected_send_size;
        self
    }

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

#[derive(Debug, Clone, PartialEq)]
pub struct CKOperation {
    configuration: CKOperationConfiguration,
    group: Option<CKOperationGroup>,
    operation_id: String,
    long_lived_operation_was_persisted: bool,
}

impl CKOperation {
    pub fn new() -> Self {
        Self {
            configuration: CKOperationConfiguration::default(),
            group: None,
            operation_id: next_identifier("operation"),
            long_lived_operation_was_persisted: false,
        }
    }

    pub const fn configuration(&self) -> &CKOperationConfiguration {
        &self.configuration
    }

    pub const fn group(&self) -> Option<&CKOperationGroup> {
        self.group.as_ref()
    }

    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    pub const fn long_lived_operation_was_persisted(&self) -> bool {
        self.long_lived_operation_was_persisted
    }

    pub fn with_configuration(mut self, configuration: CKOperationConfiguration) -> Self {
        self.configuration = configuration;
        self
    }

    pub fn with_group(mut self, group: CKOperationGroup) -> Self {
        self.group = Some(group);
        self
    }

    pub fn mark_long_lived_operation_was_persisted(&mut self) {
        self.long_lived_operation_was_persisted = true;
    }
}

impl Default for CKOperation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKDatabaseOperation {
    operation: CKOperation,
    database: Option<CKDatabase>,
}

impl CKDatabaseOperation {
    pub fn new() -> Self {
        Self {
            operation: CKOperation::default(),
            database: None,
        }
    }

    pub const fn operation(&self) -> &CKOperation {
        &self.operation
    }

    pub const fn database(&self) -> Option<&CKDatabase> {
        self.database.as_ref()
    }

    pub fn with_operation(mut self, operation: CKOperation) -> Self {
        self.operation = operation;
        self
    }

    pub fn with_database(mut self, database: CKDatabase) -> Self {
        self.operation = self
            .operation
            .clone()
            .with_configuration(self.operation.configuration().clone().with_container(database.container().clone()));
        self.database = Some(database);
        self
    }
}

impl Default for CKDatabaseOperation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKRecordZoneFetchResult {
    pub zone_id: CKRecordZoneID,
    pub record_zone: Option<CKRecordZone>,
    pub error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchRecordZonesResult {
    pub record_zones: Vec<CKRecordZone>,
    pub results: Vec<CKRecordZoneFetchResult>,
    pub operation_error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchRecordZonesOperation {
    database_operation: CKDatabaseOperation,
    record_zone_ids: Option<Vec<CKRecordZoneID>>,
}

impl CKFetchRecordZonesOperation {
    pub fn fetch_all_record_zones_operation() -> Self {
        Self {
            database_operation: CKDatabaseOperation::default(),
            record_zone_ids: None,
        }
    }

    pub fn new(record_zone_ids: Vec<CKRecordZoneID>) -> Self {
        Self {
            database_operation: CKDatabaseOperation::default(),
            record_zone_ids: Some(record_zone_ids),
        }
    }

    pub const fn database_operation(&self) -> &CKDatabaseOperation {
        &self.database_operation
    }

    pub fn record_zone_ids(&self) -> Option<&[CKRecordZoneID]> {
        self.record_zone_ids.as_deref()
    }

    pub fn with_database_operation(mut self, database_operation: CKDatabaseOperation) -> Self {
        self.database_operation = database_operation;
        self
    }

    pub fn execute_in(&self, database: &CKDatabase) -> Result<CKFetchRecordZonesResult, CloudKitError> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct CKRecordZoneSaveResult {
    pub zone_id: CKRecordZoneID,
    pub record_zone: Option<CKRecordZone>,
    pub error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKRecordZoneDeleteResult {
    pub zone_id: CKRecordZoneID,
    pub error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModifyRecordZonesResult {
    pub saved_record_zones: Vec<CKRecordZone>,
    pub deleted_record_zone_ids: Vec<CKRecordZoneID>,
    pub save_results: Vec<CKRecordZoneSaveResult>,
    pub delete_results: Vec<CKRecordZoneDeleteResult>,
    pub operation_error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKModifyRecordZonesOperation {
    database_operation: CKDatabaseOperation,
    record_zones_to_save: Vec<CKRecordZone>,
    record_zone_ids_to_delete: Vec<CKRecordZoneID>,
}

impl CKModifyRecordZonesOperation {
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

    pub const fn database_operation(&self) -> &CKDatabaseOperation {
        &self.database_operation
    }

    pub fn record_zones_to_save(&self) -> &[CKRecordZone] {
        &self.record_zones_to_save
    }

    pub fn record_zone_ids_to_delete(&self) -> &[CKRecordZoneID] {
        &self.record_zone_ids_to_delete
    }

    pub fn with_database_operation(mut self, database_operation: CKDatabaseOperation) -> Self {
        self.database_operation = database_operation;
        self
    }

    pub fn execute_in(&self, database: &CKDatabase) -> Result<ModifyRecordZonesResult, CloudKitError> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct CKSubscriptionFetchResult {
    pub subscription_id: String,
    pub subscription: Option<CKAnySubscription>,
    pub error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchSubscriptionsResult {
    pub subscriptions: Vec<CKAnySubscription>,
    pub results: Vec<CKSubscriptionFetchResult>,
    pub operation_error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchSubscriptionsOperation {
    database_operation: CKDatabaseOperation,
    subscription_ids: Option<Vec<String>>,
}

impl CKFetchSubscriptionsOperation {
    pub fn fetch_all_subscriptions_operation() -> Self {
        Self {
            database_operation: CKDatabaseOperation::default(),
            subscription_ids: None,
        }
    }

    pub fn new(subscription_ids: Vec<String>) -> Self {
        Self {
            database_operation: CKDatabaseOperation::default(),
            subscription_ids: Some(subscription_ids),
        }
    }

    pub const fn database_operation(&self) -> &CKDatabaseOperation {
        &self.database_operation
    }

    pub fn subscription_ids(&self) -> Option<&[String]> {
        self.subscription_ids.as_deref()
    }

    pub fn with_database_operation(mut self, database_operation: CKDatabaseOperation) -> Self {
        self.database_operation = database_operation;
        self
    }

    pub fn execute_in(&self, database: &CKDatabase) -> Result<CKFetchSubscriptionsResult, CloudKitError> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct CKSubscriptionSaveResult {
    pub subscription_id: String,
    pub subscription: Option<CKAnySubscription>,
    pub error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSubscriptionDeleteResult {
    pub subscription_id: String,
    pub error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModifySubscriptionsResult {
    pub saved_subscriptions: Vec<CKAnySubscription>,
    pub deleted_subscription_ids: Vec<String>,
    pub save_results: Vec<CKSubscriptionSaveResult>,
    pub delete_results: Vec<CKSubscriptionDeleteResult>,
    pub operation_error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKModifySubscriptionsOperation {
    database_operation: CKDatabaseOperation,
    subscriptions_to_save: Vec<CKAnySubscription>,
    subscription_ids_to_delete: Vec<String>,
}

impl CKModifySubscriptionsOperation {
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

    pub const fn database_operation(&self) -> &CKDatabaseOperation {
        &self.database_operation
    }

    pub fn subscriptions_to_save(&self) -> &[CKAnySubscription] {
        &self.subscriptions_to_save
    }

    pub fn subscription_ids_to_delete(&self) -> &[String] {
        &self.subscription_ids_to_delete
    }

    pub fn with_database_operation(mut self, database_operation: CKDatabaseOperation) -> Self {
        self.database_operation = database_operation;
        self
    }

    pub fn execute_in(&self, database: &CKDatabase) -> Result<ModifySubscriptionsResult, CloudKitError> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchWebAuthTokenOperation {
    database_operation: CKDatabaseOperation,
    api_token: Option<String>,
}

impl CKFetchWebAuthTokenOperation {
    pub fn new() -> Self {
        Self {
            database_operation: CKDatabaseOperation::default(),
            api_token: None,
        }
    }

    pub const fn database_operation(&self) -> &CKDatabaseOperation {
        &self.database_operation
    }

    pub fn api_token(&self) -> Option<&str> {
        self.api_token.as_deref()
    }

    pub fn with_database_operation(mut self, database_operation: CKDatabaseOperation) -> Self {
        self.database_operation = database_operation;
        self
    }

    pub fn with_api_token(mut self, api_token: impl Into<String>) -> Self {
        self.api_token = Some(api_token.into());
        self
    }

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
            return Err(unsafe { error_from_status(status, out_error) });
        }
        unsafe { parse_json_ptr::<String>(out_json, "web auth token") }
    }
}

impl Default for CKFetchWebAuthTokenOperation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKShareParticipantFetchResult {
    pub lookup_info: CKUserIdentityLookupInfo,
    pub participant: Option<CKShareParticipant>,
    pub error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchShareParticipantsResult {
    pub participants: Vec<CKShareParticipant>,
    pub results: Vec<CKShareParticipantFetchResult>,
    pub operation_error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchShareParticipantsOperation {
    operation: CKOperation,
    user_identity_lookup_infos: Vec<CKUserIdentityLookupInfo>,
}

impl CKFetchShareParticipantsOperation {
    pub fn new() -> Self {
        Self {
            operation: CKOperation::default(),
            user_identity_lookup_infos: Vec::new(),
        }
    }

    pub const fn operation(&self) -> &CKOperation {
        &self.operation
    }

    pub fn user_identity_lookup_infos(&self) -> &[CKUserIdentityLookupInfo] {
        &self.user_identity_lookup_infos
    }

    pub fn with_operation(mut self, operation: CKOperation) -> Self {
        self.operation = operation;
        self
    }

    pub fn with_user_identity_lookup_infos(
        mut self,
        user_identity_lookup_infos: Vec<CKUserIdentityLookupInfo>,
    ) -> Self {
        self.user_identity_lookup_infos = user_identity_lookup_infos;
        self
    }

    pub fn execute_in(&self, container: &CKContainer) -> Result<CKFetchShareParticipantsResult, CloudKitError> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct CKShareMetadataFetchResult {
    pub share_url: String,
    pub share_metadata: Option<CKShareMetadata>,
    pub error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchShareMetadataResult {
    pub share_metadatas: Vec<CKShareMetadata>,
    pub results: Vec<CKShareMetadataFetchResult>,
    pub operation_error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKFetchShareMetadataOperation {
    operation: CKOperation,
    share_urls: Vec<String>,
    should_fetch_root_record: bool,
    root_record_desired_keys: Option<Vec<String>>,
}

impl CKFetchShareMetadataOperation {
    pub fn new() -> Self {
        Self {
            operation: CKOperation::default(),
            share_urls: Vec::new(),
            should_fetch_root_record: false,
            root_record_desired_keys: None,
        }
    }

    pub const fn operation(&self) -> &CKOperation {
        &self.operation
    }

    pub fn share_urls(&self) -> &[String] {
        &self.share_urls
    }

    pub const fn should_fetch_root_record(&self) -> bool {
        self.should_fetch_root_record
    }

    pub fn root_record_desired_keys(&self) -> Option<&[String]> {
        self.root_record_desired_keys.as_deref()
    }

    pub fn with_operation(mut self, operation: CKOperation) -> Self {
        self.operation = operation;
        self
    }

    pub fn with_share_urls(mut self, share_urls: Vec<String>) -> Self {
        self.share_urls = share_urls;
        self
    }

    pub fn with_should_fetch_root_record(mut self, should_fetch_root_record: bool) -> Self {
        self.should_fetch_root_record = should_fetch_root_record;
        self
    }

    pub fn with_root_record_desired_keys(mut self, root_record_desired_keys: Vec<String>) -> Self {
        self.root_record_desired_keys = Some(root_record_desired_keys);
        self
    }

    pub fn execute_in(&self, container: &CKContainer) -> Result<CKFetchShareMetadataResult, CloudKitError> {
        let identifier = optional_cstring_from_str(
            container.container_identifier(),
            "container identifier",
        )?;
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

#[derive(Debug, Clone, PartialEq)]
pub struct CKAcceptShareResult {
    pub share_metadata: CKShareMetadata,
    pub accepted_share: Option<CKShare>,
    pub error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKAcceptSharesResult {
    pub accepted_shares: Vec<CKShare>,
    pub results: Vec<CKAcceptShareResult>,
    pub operation_error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKAcceptSharesOperation {
    operation: CKOperation,
    share_metadatas: Vec<CKShareMetadata>,
}

impl CKAcceptSharesOperation {
    pub fn new() -> Self {
        Self {
            operation: CKOperation::default(),
            share_metadatas: Vec::new(),
        }
    }

    pub const fn operation(&self) -> &CKOperation {
        &self.operation
    }

    pub fn share_metadatas(&self) -> &[CKShareMetadata] {
        &self.share_metadatas
    }

    pub fn with_operation(mut self, operation: CKOperation) -> Self {
        self.operation = operation;
        self
    }

    pub fn with_share_metadatas(mut self, share_metadatas: Vec<CKShareMetadata>) -> Self {
        self.share_metadatas = share_metadatas;
        self
    }

    pub fn execute_in(&self, container: &CKContainer) -> Result<CKAcceptSharesResult, CloudKitError> {
        let identifier = optional_cstring_from_str(
            container.container_identifier(),
            "container identifier",
        )?;
        let mut accepted_shares = Vec::new();
        let mut results = Vec::new();
        for share_metadata in &self.share_metadatas {
            let share_metadata_json = json_cstring(&share_metadata.to_payload(), "share metadata")?;
            let mut out_json: *mut c_char = ptr::null_mut();
            let mut out_error: *mut c_char = ptr::null_mut();
            let status = unsafe {
                ffi::ck_container_accept_share_metadata_sync(
                    opt_cstring_ptr(&identifier),
                    share_metadata_json.as_ptr(),
                    &mut out_json,
                    &mut out_error,
                )
            };
            if status == ffi::status::OK {
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

#[derive(Debug, Clone, PartialEq)]
pub struct CKShareAccessRequestResult {
    pub share_url: String,
    pub error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKShareRequestAccessResult {
    pub results: Vec<CKShareAccessRequestResult>,
    pub operation_error: Option<CloudKitError>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKShareRequestAccessOperation {
    operation: CKOperation,
    share_urls: Vec<String>,
}

impl CKShareRequestAccessOperation {
    pub fn new() -> Self {
        Self {
            operation: CKOperation::default(),
            share_urls: Vec::new(),
        }
    }

    pub const fn operation(&self) -> &CKOperation {
        &self.operation
    }

    pub fn share_urls(&self) -> &[String] {
        &self.share_urls
    }

    pub fn with_operation(mut self, operation: CKOperation) -> Self {
        self.operation = operation;
        self
    }

    pub fn with_share_urls(mut self, share_urls: Vec<String>) -> Self {
        self.share_urls = share_urls;
        self
    }

    pub fn execute_in(&self, container: &CKContainer) -> Result<CKShareRequestAccessResult, CloudKitError> {
        let identifier = optional_cstring_from_str(
            container.container_identifier(),
            "container identifier",
        )?;
        let mut results = Vec::new();
        for share_url in &self.share_urls {
            let share_url = cstring_from_str(share_url, "share URL")?;
            let mut out_error: *mut c_char = ptr::null_mut();
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
