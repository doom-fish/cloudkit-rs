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

pub mod asset;
pub mod container;
pub mod database;
pub mod error;
pub mod fetched_results;
pub mod ffi;
pub mod notification_info;
pub mod operation;
mod private;
pub mod query;
pub mod record;
pub mod record_id;
pub mod reference_utility;
pub mod server_change_token;
pub mod share;
pub mod subscription;
pub mod user_identity;
pub mod zone;

pub use asset::CKAsset;
pub use container::{AccountStatus, CKContainer};
pub use database::{CKDatabase, CKDatabaseScope};
pub use error::{
    CloudKitError, CloudKitErrorCode, CLOUDKIT_BRIDGE_ERROR_DOMAIN, CLOUDKIT_ERROR_DOMAIN,
};
pub use fetched_results::{
    CKDeletedRecord, CKFetchDatabaseChangesResult, CKFetchRecordZoneChangesResult,
    CKFetchRecordZoneResult, CKFetchRecordsResult, CKFetchedQueryResults, CKQueryCursor,
    CKRecordResult,
};
pub use notification_info::CKNotificationInfo;
pub use operation::{
    CKFetchDatabaseChangesOperation, CKFetchRecordZoneChangesConfiguration,
    CKFetchRecordZoneChangesOperation, CKFetchRecordsOperation, CKModifyRecordsOperation,
    CKQueryOperation, CKRecordDeleteResult, CKRecordSavePolicy, CKRecordSaveResult,
    ModifyRecordsResult, QueryMatchResult, QueryOperationResult,
};
pub use query::{CKQuery, SortDescriptor};
pub use record::{CKRecord, RecordValue};
pub use record_id::CKRecordID;
pub use reference_utility::{CKReference, CKReferenceAction};
pub use server_change_token::CKServerChangeToken;
pub use share::{
    CKShare, CKShareParticipant, CKShareParticipantAcceptanceStatus,
    CKShareParticipantPermission, CKShareParticipantRole,
};
pub use subscription::{
    CKAnySubscription, CKDatabaseSubscription, CKQuerySubscription, CKRecordZoneSubscription,
    CKSubscription, CKSubscriptionType, QuerySubscriptionOptions,
};
pub use user_identity::{CKPersonNameComponents, CKUserIdentity, CKUserIdentityLookupInfo};
pub use zone::{
    CKRecordZone, CKRecordZoneCapabilities, CKRecordZoneEncryptionScope, CKRecordZoneID,
};

/// Common imports.
pub mod prelude {
    pub use crate::asset::CKAsset;
    pub use crate::container::{AccountStatus, CKContainer};
    pub use crate::database::{CKDatabase, CKDatabaseScope};
    pub use crate::error::{
        CloudKitError, CloudKitErrorCode, CLOUDKIT_BRIDGE_ERROR_DOMAIN, CLOUDKIT_ERROR_DOMAIN,
    };
    pub use crate::fetched_results::{
        CKDeletedRecord, CKFetchDatabaseChangesResult, CKFetchRecordZoneChangesResult,
        CKFetchRecordZoneResult, CKFetchRecordsResult, CKFetchedQueryResults, CKQueryCursor,
        CKRecordResult,
    };
    pub use crate::notification_info::CKNotificationInfo;
    pub use crate::operation::{
        CKFetchDatabaseChangesOperation, CKFetchRecordZoneChangesConfiguration,
        CKFetchRecordZoneChangesOperation, CKFetchRecordsOperation, CKModifyRecordsOperation,
        CKQueryOperation, CKRecordDeleteResult, CKRecordSavePolicy, CKRecordSaveResult,
        ModifyRecordsResult, QueryMatchResult, QueryOperationResult,
    };
    pub use crate::query::{CKQuery, SortDescriptor};
    pub use crate::record::{CKRecord, RecordValue};
    pub use crate::record_id::CKRecordID;
    pub use crate::reference_utility::{CKReference, CKReferenceAction};
    pub use crate::server_change_token::CKServerChangeToken;
    pub use crate::share::{
        CKShare, CKShareParticipant, CKShareParticipantAcceptanceStatus,
        CKShareParticipantPermission, CKShareParticipantRole,
    };
    pub use crate::subscription::{
        CKAnySubscription, CKDatabaseSubscription, CKQuerySubscription,
        CKRecordZoneSubscription, CKSubscription, CKSubscriptionType,
        QuerySubscriptionOptions,
    };
    pub use crate::user_identity::{
        CKPersonNameComponents, CKUserIdentity, CKUserIdentityLookupInfo,
    };
    pub use crate::zone::{
        CKRecordZone, CKRecordZoneCapabilities, CKRecordZoneEncryptionScope, CKRecordZoneID,
    };
}
