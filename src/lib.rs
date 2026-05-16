#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(
    clippy::cargo_common_metadata,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::redundant_pub_crate,
    clippy::ref_option,
    clippy::return_self_not_must_use,
    clippy::should_implement_trait,
    clippy::struct_excessive_bools,
    clippy::use_self
)]

pub mod container;
pub mod database;
pub mod error;
pub mod ffi;
pub mod operation;
mod private;
pub mod query;
pub mod record;
pub mod subscription;

pub use container::{AccountStatus, CKContainer};
pub use database::{CKDatabase, CKDatabaseScope};
pub use error::{
    CloudKitError, CloudKitErrorCode, CLOUDKIT_BRIDGE_ERROR_DOMAIN, CLOUDKIT_ERROR_DOMAIN,
};
pub use operation::{
    CKModifyRecordsOperation, CKQueryOperation, CKRecordDeleteResult, CKRecordSavePolicy,
    CKRecordSaveResult, ModifyRecordsResult, QueryMatchResult, QueryOperationResult,
};
pub use query::{CKQuery, SortDescriptor};
pub use record::{CKAsset, CKRecord, CKRecordID, CKRecordZone, CKRecordZoneID, RecordValue};
pub use subscription::{
    CKNotificationInfo, CKQuerySubscription, CKRecordZoneSubscription, CKSubscription,
    CKSubscriptionType, QuerySubscriptionOptions,
};

/// Common imports.
pub mod prelude {
    pub use crate::container::{AccountStatus, CKContainer};
    pub use crate::database::{CKDatabase, CKDatabaseScope};
    pub use crate::error::{
        CloudKitError, CloudKitErrorCode, CLOUDKIT_BRIDGE_ERROR_DOMAIN, CLOUDKIT_ERROR_DOMAIN,
    };
    pub use crate::operation::{
        CKModifyRecordsOperation, CKQueryOperation, CKRecordDeleteResult, CKRecordSavePolicy,
        CKRecordSaveResult, ModifyRecordsResult, QueryMatchResult, QueryOperationResult,
    };
    pub use crate::query::{CKQuery, SortDescriptor};
    pub use crate::record::{
        CKAsset, CKRecord, CKRecordID, CKRecordZone, CKRecordZoneID, RecordValue,
    };
    pub use crate::subscription::{
        CKNotificationInfo, CKQuerySubscription, CKRecordZoneSubscription, CKSubscription,
        CKSubscriptionType, QuerySubscriptionOptions,
    };
}
