#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::cargo_common_metadata,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::derive_partial_eq_without_eq,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::redundant_pub_crate,
    clippy::ref_option,
    clippy::return_self_not_must_use,
    clippy::should_implement_trait,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::use_self
)]

/// Exposes `CloudKit` asset wrappers.
pub mod asset;
/// Exposes async `CloudKit` wrappers.
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod async_api;
/// Exposes `CloudKit` framework constants.
pub mod constants;
/// Exposes `CloudKit` container wrappers.
pub mod container;
/// Exposes `CloudKit` database wrappers.
pub mod database;
/// Exposes `CloudKit` error wrappers.
pub mod error;
/// Exposes `CloudKit` fetched-result wrappers.
pub mod fetched_results;
/// Exposes the low-level `CloudKit` bridge module.
pub mod ffi;
/// Exposes `CloudKit` notification wrappers.
pub mod notification;
/// Exposes `CloudKit` notification-info wrappers.
pub mod notification_info;
/// Exposes `CloudKit` operation wrappers.
pub mod operation;
mod private;
/// Exposes `CloudKit` query wrappers.
pub mod query;
/// Exposes `CloudKit` record wrappers.
pub mod record;
/// Exposes `CloudKit` record ID wrappers.
pub mod record_id;
/// Exposes `CloudKit` reference wrappers.
pub mod reference_utility;
/// Exposes `CloudKit` server-change-token wrappers.
pub mod server_change_token;
/// Exposes `CloudKit` sharing wrappers.
pub mod share;
/// Exposes `CloudKit` subscription wrappers.
pub mod subscription;
/// Exposes `CloudKit` sync-engine wrappers.
pub mod sync_engine;
/// Exposes `CloudKit` user-identity wrappers.
pub mod user_identity;
/// Exposes `CloudKit` record-zone wrappers.
pub mod zone;

pub use asset::CKAsset;
pub use constants::*;
pub use container::{
    AccountStatus, CKApplicationPermissionStatus, CKApplicationPermissions, CKContainer,
};
pub use database::{CKDatabase, CKDatabaseScope};
pub use error::{
    CloudKitError, CloudKitErrorCode, CLOUDKIT_BRIDGE_ERROR_DOMAIN, CLOUDKIT_ERROR_DOMAIN,
};
pub use fetched_results::{
    CKDeletedRecord, CKFetchDatabaseChangesResult, CKFetchRecordZoneChangesResult,
    CKFetchRecordZoneResult, CKFetchRecordsResult, CKFetchedQueryResults, CKQueryCursor,
    CKRecordResult,
};
pub use notification::*;
pub use notification_info::CKNotificationInfo;
pub use operation::*;
pub use query::{CKLocationSortDescriptor, CKQuery, SortDescriptor};
pub use record::{CKRecord, CKRecordKeyValueSetting, RecordValue};
pub use record_id::CKRecordID;
pub use reference_utility::{CKReference, CKReferenceAction};
pub use server_change_token::CKServerChangeToken;
pub use share::*;
pub use subscription::{
    CKAnySubscription, CKDatabaseSubscription, CKQuerySubscription, CKRecordZoneSubscription,
    CKSubscription, CKSubscriptionType, QuerySubscriptionOptions,
};
pub use sync_engine::*;
pub use user_identity::{CKPersonNameComponents, CKUserIdentity, CKUserIdentityLookupInfo};
pub use zone::{
    CKRecordZone, CKRecordZoneCapabilities, CKRecordZoneEncryptionScope, CKRecordZoneID,
};

/// Common imports.
pub mod prelude {
    pub use crate::asset::CKAsset;
    pub use crate::constants::*;
    pub use crate::container::{
        AccountStatus, CKApplicationPermissionStatus, CKApplicationPermissions, CKContainer,
    };
    pub use crate::database::{CKDatabase, CKDatabaseScope};
    pub use crate::error::{
        CloudKitError, CloudKitErrorCode, CLOUDKIT_BRIDGE_ERROR_DOMAIN, CLOUDKIT_ERROR_DOMAIN,
    };
    pub use crate::fetched_results::{
        CKDeletedRecord, CKFetchDatabaseChangesResult, CKFetchRecordZoneChangesResult,
        CKFetchRecordZoneResult, CKFetchRecordsResult, CKFetchedQueryResults, CKQueryCursor,
        CKRecordResult,
    };
    pub use crate::notification::*;
    pub use crate::notification_info::CKNotificationInfo;
    pub use crate::operation::*;
    pub use crate::query::{CKLocationSortDescriptor, CKQuery, SortDescriptor};
    pub use crate::record::{CKRecord, CKRecordKeyValueSetting, RecordValue};
    pub use crate::record_id::CKRecordID;
    pub use crate::reference_utility::{CKReference, CKReferenceAction};
    pub use crate::server_change_token::CKServerChangeToken;
    pub use crate::share::*;
    pub use crate::subscription::{
        CKAnySubscription, CKDatabaseSubscription, CKQuerySubscription, CKRecordZoneSubscription,
        CKSubscription, CKSubscriptionType, QuerySubscriptionOptions,
    };
    pub use crate::sync_engine::*;
    pub use crate::user_identity::{
        CKPersonNameComponents, CKUserIdentity, CKUserIdentityLookupInfo,
    };
    pub use crate::zone::{
        CKRecordZone, CKRecordZoneCapabilities, CKRecordZoneEncryptionScope, CKRecordZoneID,
    };
}
