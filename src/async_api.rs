//! Async API for `cloudkit`
//!
//! Gated behind the `async` cargo feature. These wrappers expose `Future`-based
//! variants of selected `CKContainer`, `CKDatabase`, and operation APIs.

use core::ffi::{c_char, c_void};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use doom_fish_utils::completion::{error_from_cstr, AsyncCompletion, AsyncCompletionFuture};
use doom_fish_utils::panic_safe::catch_user_panic;
use serde::de::DeserializeOwned;

use crate::container::{
    AccountStatus, CKApplicationPermissionStatus, CKApplicationPermissions, CKContainer,
};
use crate::database::CKDatabase;
use crate::error::{CloudKitError, ErrorPayload};
use crate::fetched_results::{CKFetchDatabaseChangesResult, CKFetchedQueryResults};
use crate::ffi;
use crate::operation::{
    CKFetchDatabaseChangesOperation, CKModifyRecordsOperation, CKRecordDeleteResult,
    CKRecordSaveResult, ModifyRecordsResult,
};
use crate::private::{
    json_cstring, opt_cstring_ptr, optional_cstring_from_str, parse_error_json_str, parse_json_str,
    CKFetchDatabaseChangesOperationPayload, CKModifyRecordsOperationPayload,
    CKModifyRecordsResultPayload, CKRecordPayload, CKRecordZonePayload,
};
use crate::query::CKQuery;
use crate::record::{CKRecord, CKRecordID, CKRecordZone, CKRecordZoneID};
use crate::server_change_token::CKServerChangeToken;
use crate::user_identity::{CKUserIdentity, CKUserIdentityLookupInfo};

fn encode_async_error(error: CloudKitError) -> String {
    serde_json::to_string(&ErrorPayload {
        domain: error.domain,
        code: error.code,
        message: error.message,
        retry_after_seconds: error.retry_after_seconds,
    })
    .unwrap_or_else(|_| {
        "{\"domain\":\"CloudKitBridge\",\"code\":-2,\"message\":\"CloudKit bridge returned an unencodable async error\",\"retryAfterSeconds\":null}".into()
    })
}

fn bridge_error_json(code: i64, message: impl Into<String>) -> String {
    encode_async_error(CloudKitError::bridge(code, message))
}

fn ready_error_future<T>(error: CloudKitError) -> AsyncCompletionFuture<T> {
    let (future, context) = AsyncCompletion::create();
    // SAFETY: `context` is a valid `SyncCompletionPtr` from `AsyncCompletion::create()`, not yet consumed.
    unsafe { AsyncCompletion::<T>::complete_err(context, encode_async_error(error)) };
    future
}

fn poll_status_future<T>(
    inner: &mut AsyncCompletionFuture<i32>,
    cx: &mut Context<'_>,
    map_ok: fn(i32) -> T,
) -> Poll<Result<T, CloudKitError>> {
    Pin::new(inner).poll(cx).map(|result| {
        result
            .map(map_ok)
            .map_err(|message| parse_error_json_str(&message))
    })
}

fn poll_json_future<T, P>(
    inner: &mut AsyncCompletionFuture<String>,
    cx: &mut Context<'_>,
    context: &'static str,
    map_ok: fn(P) -> T,
) -> Poll<Result<T, CloudKitError>>
where
    P: DeserializeOwned,
{
    Pin::new(inner).poll(cx).map(|result| {
        result
            .map_err(|message| parse_error_json_str(&message))
            .and_then(|json| parse_json_str::<P>(&json, context))
            .map(map_ok)
    })
}

fn records_from_payloads(payloads: Vec<CKRecordPayload>) -> Vec<CKRecord> {
    payloads.into_iter().map(CKRecord::from_payload).collect()
}

fn record_zones_from_payloads(payloads: Vec<CKRecordZonePayload>) -> Vec<CKRecordZone> {
    payloads
        .into_iter()
        .map(CKRecordZone::from_payload)
        .collect()
}

fn modify_records_result_from_payload(
    payload: CKModifyRecordsResultPayload,
) -> ModifyRecordsResult {
    ModifyRecordsResult {
        saved_records: payload
            .saved_records
            .into_iter()
            .map(CKRecord::from_payload)
            .collect(),
        deleted_record_ids: payload
            .deleted_record_ids
            .into_iter()
            .map(CKRecordID::from_payload)
            .collect(),
        save_results: payload
            .save_results
            .into_iter()
            .map(|entry| CKRecordSaveResult {
                record_id: CKRecordID::from_payload(entry.record_id),
                record: entry.record.map(CKRecord::from_payload),
                error: entry.error.map(CloudKitError::from_payload),
            })
            .collect(),
        delete_results: payload
            .delete_results
            .into_iter()
            .map(|entry| CKRecordDeleteResult {
                record_id: CKRecordID::from_payload(entry.record_id),
                error: entry.error.map(CloudKitError::from_payload),
            })
            .collect(),
        operation_error: payload.operation_error.map(CloudKitError::from_payload),
    }
}

extern "C" fn status_callback(refcon: *mut c_void, status_raw: i32, error_json: *const c_char) {
    catch_user_panic("cloudkit::status_callback", || {
        if error_json.is_null() {
            // SAFETY: `refcon` is a valid `SyncCompletionPtr` from `AsyncCompletion::create()`, not yet consumed.
            unsafe { AsyncCompletion::<i32>::complete_ok(refcon, status_raw) };
        } else {
            let error = unsafe { error_from_cstr(error_json.cast()) };
            // SAFETY: `refcon` is a valid `SyncCompletionPtr` from `AsyncCompletion::create()`, not yet consumed.
            unsafe { AsyncCompletion::<i32>::complete_err(refcon, error) };
        }
    });
}

extern "C" fn json_callback(refcon: *mut c_void, json: *const c_char, error_json: *const c_char) {
    catch_user_panic("cloudkit::json_callback", || {
        if !error_json.is_null() {
            let error = unsafe { error_from_cstr(error_json.cast()) };
            // SAFETY: `refcon` is a valid `SyncCompletionPtr` from `AsyncCompletion::create()`, not yet consumed.
            unsafe { AsyncCompletion::<String>::complete_err(refcon, error) };
        } else if !json.is_null() {
            let json_str = unsafe { error_from_cstr(json.cast()) };
            // SAFETY: `refcon` is a valid `SyncCompletionPtr` from `AsyncCompletion::create()`, not yet consumed.
            unsafe { AsyncCompletion::<String>::complete_ok(refcon, json_str) };
        } else {
            // SAFETY: `refcon` is a valid `SyncCompletionPtr` from `AsyncCompletion::create()`, not yet consumed.
            unsafe {
                AsyncCompletion::<String>::complete_err(
                    refcon,
                    bridge_error_json(-2, "CloudKit bridge returned an empty JSON payload"),
                );
            };
        }
    });
}

/// Future resolving to the iCloud account status.
pub struct AccountStatusFuture {
    inner: AsyncCompletionFuture<i32>,
}

impl Future for AccountStatusFuture {
    type Output = Result<AccountStatus, CloudKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_status_future(&mut self.inner, cx, AccountStatus::from_raw)
    }
}

/// Future resolving to the current user's `CloudKit` record ID.
pub struct FetchUserRecordIdFuture {
    inner: AsyncCompletionFuture<String>,
}

impl Future for FetchUserRecordIdFuture {
    type Output = Result<CKRecordID, CloudKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_json_future(
            &mut self.inner,
            cx,
            "user record ID",
            CKRecordID::from_payload,
        )
    }
}

/// Future resolving to the application permission status after prompting the user.
pub struct RequestApplicationPermissionFuture {
    inner: AsyncCompletionFuture<i32>,
}

impl Future for RequestApplicationPermissionFuture {
    type Output = Result<CKApplicationPermissionStatus, CloudKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_status_future(&mut self.inner, cx, CKApplicationPermissionStatus::from_raw)
    }
}

/// Future resolving to a discovered user identity.
pub struct DiscoverUserIdentityFuture {
    inner: AsyncCompletionFuture<String>,
}

impl Future for DiscoverUserIdentityFuture {
    type Output = Result<CKUserIdentity, CloudKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_json_future(
            &mut self.inner,
            cx,
            "user identity",
            CKUserIdentity::from_payload,
        )
    }
}

/// Future resolving to query result records.
pub struct PerformQueryFuture {
    inner: AsyncCompletionFuture<String>,
}

impl Future for PerformQueryFuture {
    type Output = Result<Vec<CKRecord>, CloudKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_json_future(&mut self.inner, cx, "query results", records_from_payloads)
    }
}

/// Future resolving to a single fetched record.
pub struct FetchRecordFuture {
    inner: AsyncCompletionFuture<String>,
}

impl Future for FetchRecordFuture {
    type Output = Result<CKRecord, CloudKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_json_future(
            &mut self.inner,
            cx,
            "fetched record",
            CKRecord::from_payload,
        )
    }
}

/// Future resolving to the result of a modify-records operation.
pub struct ModifyRecordsFuture {
    inner: AsyncCompletionFuture<String>,
}

impl Future for ModifyRecordsFuture {
    type Output = Result<ModifyRecordsResult, CloudKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_json_future(
            &mut self.inner,
            cx,
            "modify-records result",
            modify_records_result_from_payload,
        )
    }
}

/// Future resolving to the record ID of a deleted record.
pub struct DeleteRecordFuture {
    inner: AsyncCompletionFuture<String>,
}

impl Future for DeleteRecordFuture {
    type Output = Result<CKRecordID, CloudKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_json_future(
            &mut self.inner,
            cx,
            "deleted record ID",
            CKRecordID::from_payload,
        )
    }
}

/// Future resolving to paged query results with an optional continuation cursor.
pub struct FetchQueryResultsFuture {
    inner: AsyncCompletionFuture<String>,
}

impl Future for FetchQueryResultsFuture {
    type Output = Result<CKFetchedQueryResults, CloudKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_json_future(
            &mut self.inner,
            cx,
            "fetched query results",
            CKFetchedQueryResults::from_payload,
        )
    }
}

/// Future resolving to all database-change records since a server change token.
pub struct FetchDatabaseChangesFuture {
    inner: AsyncCompletionFuture<String>,
}

impl Future for FetchDatabaseChangesFuture {
    type Output = Result<CKFetchDatabaseChangesResult, CloudKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_json_future(
            &mut self.inner,
            cx,
            "fetch-database-changes result",
            CKFetchDatabaseChangesResult::from_payload,
        )
    }
}

/// Future resolving to all record zones in a database.
pub struct FetchAllRecordZonesFuture {
    inner: AsyncCompletionFuture<String>,
}

impl Future for FetchAllRecordZonesFuture {
    type Output = Result<Vec<CKRecordZone>, CloudKitError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_json_future(
            &mut self.inner,
            cx,
            "record zones",
            record_zones_from_payloads,
        )
    }
}

impl core::fmt::Debug for AccountStatusFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AccountStatusFuture")
            .finish_non_exhaustive()
    }
}

impl core::fmt::Debug for FetchUserRecordIdFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FetchUserRecordIdFuture")
            .finish_non_exhaustive()
    }
}

impl core::fmt::Debug for RequestApplicationPermissionFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RequestApplicationPermissionFuture")
            .finish_non_exhaustive()
    }
}

impl core::fmt::Debug for DiscoverUserIdentityFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DiscoverUserIdentityFuture")
            .finish_non_exhaustive()
    }
}

impl core::fmt::Debug for PerformQueryFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PerformQueryFuture").finish_non_exhaustive()
    }
}

impl core::fmt::Debug for FetchRecordFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FetchRecordFuture").finish_non_exhaustive()
    }
}

impl core::fmt::Debug for ModifyRecordsFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ModifyRecordsFuture")
            .finish_non_exhaustive()
    }
}

impl core::fmt::Debug for DeleteRecordFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DeleteRecordFuture").finish_non_exhaustive()
    }
}

impl core::fmt::Debug for FetchQueryResultsFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FetchQueryResultsFuture")
            .finish_non_exhaustive()
    }
}

impl core::fmt::Debug for FetchDatabaseChangesFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FetchDatabaseChangesFuture")
            .finish_non_exhaustive()
    }
}

impl core::fmt::Debug for FetchAllRecordZonesFuture {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FetchAllRecordZonesFuture")
            .finish_non_exhaustive()
    }
}

impl CKContainer {
    /// Wraps the async form of `CKContainer.accountStatus`.
    pub fn account_status_async(&self) -> AccountStatusFuture {
        let identifier =
            match optional_cstring_from_str(self.container_identifier(), "container identifier") {
                Ok(identifier) => identifier,
                Err(error) => {
                    return AccountStatusFuture {
                        inner: ready_error_future(error),
                    };
                }
            };
        let (future, context) = AsyncCompletion::create();
        // SAFETY: all C-string arguments are valid for the duration of the call; `context` is a valid `SyncCompletionPtr` consumed exactly once by the callback.
        unsafe {
            ffi::ck_container_account_status_async(
                opt_cstring_ptr(&identifier),
                status_callback,
                context,
            );
        };
        AccountStatusFuture { inner: future }
    }

    /// Wraps the async form of `CKContainer.fetchUserRecordID`.
    pub fn fetch_user_record_id_async(&self) -> FetchUserRecordIdFuture {
        let identifier =
            match optional_cstring_from_str(self.container_identifier(), "container identifier") {
                Ok(identifier) => identifier,
                Err(error) => {
                    return FetchUserRecordIdFuture {
                        inner: ready_error_future(error),
                    };
                }
            };
        let (future, context) = AsyncCompletion::create();
        // SAFETY: all C-string arguments are valid for the duration of the call; `context` is a valid `SyncCompletionPtr` consumed exactly once by the callback.
        unsafe {
            ffi::ck_container_fetch_user_record_id_async(
                opt_cstring_ptr(&identifier),
                json_callback,
                context,
            );
        };
        FetchUserRecordIdFuture { inner: future }
    }

    /// Wraps the async form of `CKContainer.requestApplicationPermission(_:)`.
    pub fn request_application_permission_async(
        &self,
        permission: CKApplicationPermissions,
    ) -> RequestApplicationPermissionFuture {
        let identifier =
            match optional_cstring_from_str(self.container_identifier(), "container identifier") {
                Ok(identifier) => identifier,
                Err(error) => {
                    return RequestApplicationPermissionFuture {
                        inner: ready_error_future(error),
                    };
                }
            };
        let Ok(permission_raw) = i32::try_from(permission.bits()) else {
            return RequestApplicationPermissionFuture {
                inner: ready_error_future(CloudKitError::bridge(
                    -1,
                    "application permission bits exceed supported i32 range",
                )),
            };
        };
        let (future, context) = AsyncCompletion::create();
        // SAFETY: all C-string arguments are valid for the duration of the call; `context` is a valid `SyncCompletionPtr` consumed exactly once by the callback.
        unsafe {
            ffi::ck_container_request_application_permission_async(
                opt_cstring_ptr(&identifier),
                permission_raw,
                status_callback,
                context,
            );
        };
        RequestApplicationPermissionFuture { inner: future }
    }

    /// Wraps the async form of `CKContainer.discoverUserIdentity(with:)`.
    pub fn discover_user_identity_async(
        &self,
        lookup_info: &CKUserIdentityLookupInfo,
    ) -> DiscoverUserIdentityFuture {
        let identifier =
            match optional_cstring_from_str(self.container_identifier(), "container identifier") {
                Ok(identifier) => identifier,
                Err(error) => {
                    return DiscoverUserIdentityFuture {
                        inner: ready_error_future(error),
                    };
                }
            };
        let lookup_json = match json_cstring(&lookup_info.to_payload(), "user identity lookup info")
        {
            Ok(lookup_json) => lookup_json,
            Err(error) => {
                return DiscoverUserIdentityFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let (future, context) = AsyncCompletion::create();
        // SAFETY: all C-string arguments are valid for the duration of the call; `context` is a valid `SyncCompletionPtr` consumed exactly once by the callback.
        unsafe {
            ffi::ck_container_discover_user_identity_async(
                opt_cstring_ptr(&identifier),
                lookup_json.as_ptr(),
                json_callback,
                context,
            );
        };
        DiscoverUserIdentityFuture { inner: future }
    }

    /// Wraps the email-address form of `CKContainer.discoverUserIdentity(with:)`.
    pub fn discover_user_identity_with_email_address_async(
        &self,
        email_address: impl Into<String>,
    ) -> DiscoverUserIdentityFuture {
        self.discover_user_identity_async(&CKUserIdentityLookupInfo::with_email_address(
            email_address,
        ))
    }

    /// Wraps the phone-number form of `CKContainer.discoverUserIdentity(with:)`.
    pub fn discover_user_identity_with_phone_number_async(
        &self,
        phone_number: impl Into<String>,
    ) -> DiscoverUserIdentityFuture {
        self.discover_user_identity_async(&CKUserIdentityLookupInfo::with_phone_number(
            phone_number,
        ))
    }

    /// Wraps the user-record-ID form of `CKContainer.discoverUserIdentity(with:)`.
    pub fn discover_user_identity_with_user_record_id_async(
        &self,
        user_record_id: CKRecordID,
    ) -> DiscoverUserIdentityFuture {
        self.discover_user_identity_async(&CKUserIdentityLookupInfo::with_user_record_id(
            user_record_id,
        ))
    }
}

impl CKDatabase {
    /// Wraps the async form of `CKDatabase.performQuery(_:inZoneWith:)`.
    pub fn perform_query_async(
        &self,
        query: &CKQuery,
        zone_id: Option<&CKRecordZoneID>,
    ) -> PerformQueryFuture {
        let identifier = match optional_cstring_from_str(
            self.container().container_identifier(),
            "container identifier",
        ) {
            Ok(identifier) => identifier,
            Err(error) => {
                return PerformQueryFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let query_json = match json_cstring(&query.to_payload(), "query") {
            Ok(query_json) => query_json,
            Err(error) => {
                return PerformQueryFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let zone_payload = zone_id.map(CKRecordZoneID::to_payload);
        let zone_json = match zone_payload
            .as_ref()
            .map(|zone| json_cstring(zone, "zone ID"))
            .transpose()
        {
            Ok(zone_json) => zone_json,
            Err(error) => {
                return PerformQueryFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let (future, context) = AsyncCompletion::create();
        // SAFETY: all C-string arguments are valid for the duration of the call; `context` is a valid `SyncCompletionPtr` consumed exactly once by the callback.
        unsafe {
            ffi::ck_database_perform_query_async(
                opt_cstring_ptr(&identifier),
                self.database_scope() as i32,
                query_json.as_ptr(),
                opt_cstring_ptr(&zone_json),
                json_callback,
                context,
            );
        };
        PerformQueryFuture { inner: future }
    }

    /// Wraps the async form of `CKDatabase.fetchRecord(withID:)`.
    pub fn fetch_record_async(&self, record_id: &CKRecordID) -> FetchRecordFuture {
        let identifier = match optional_cstring_from_str(
            self.container().container_identifier(),
            "container identifier",
        ) {
            Ok(identifier) => identifier,
            Err(error) => {
                return FetchRecordFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let record_id_json = match json_cstring(&record_id.to_payload(), "record ID") {
            Ok(record_id_json) => record_id_json,
            Err(error) => {
                return FetchRecordFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let (future, context) = AsyncCompletion::create();
        // SAFETY: all C-string arguments are valid for the duration of the call; `context` is a valid `SyncCompletionPtr` consumed exactly once by the callback.
        unsafe {
            ffi::ck_database_fetch_record_async(
                opt_cstring_ptr(&identifier),
                self.database_scope() as i32,
                record_id_json.as_ptr(),
                json_callback,
                context,
            );
        };
        FetchRecordFuture { inner: future }
    }

    /// Wraps the async form of `CKDatabase.deleteRecord(withID:)`.
    pub fn delete_record_async(&self, record_id: &CKRecordID) -> DeleteRecordFuture {
        let identifier = match optional_cstring_from_str(
            self.container().container_identifier(),
            "container identifier",
        ) {
            Ok(identifier) => identifier,
            Err(error) => {
                return DeleteRecordFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let record_id_json = match json_cstring(&record_id.to_payload(), "record ID") {
            Ok(record_id_json) => record_id_json,
            Err(error) => {
                return DeleteRecordFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let (future, context) = AsyncCompletion::create();
        // SAFETY: all C-string arguments are valid for the duration of the call; `context` is a valid `SyncCompletionPtr` consumed exactly once by the callback.
        unsafe {
            ffi::ck_database_delete_record_async(
                opt_cstring_ptr(&identifier),
                self.database_scope() as i32,
                record_id_json.as_ptr(),
                json_callback,
                context,
            );
        };
        DeleteRecordFuture { inner: future }
    }

    /// Wraps the async form of `CKDatabase.records(matching:inZoneWith:)`.
    pub fn fetch_query_results_async(
        &self,
        query: &CKQuery,
        zone_id: Option<&CKRecordZoneID>,
        desired_keys: Option<Vec<String>>,
        results_limit: Option<usize>,
    ) -> FetchQueryResultsFuture {
        let identifier = match optional_cstring_from_str(
            self.container().container_identifier(),
            "container identifier",
        ) {
            Ok(identifier) => identifier,
            Err(error) => {
                return FetchQueryResultsFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let query_json = match json_cstring(&query.to_payload(), "query") {
            Ok(query_json) => query_json,
            Err(error) => {
                return FetchQueryResultsFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let zone_payload = zone_id.map(CKRecordZoneID::to_payload);
        let zone_json = match zone_payload
            .as_ref()
            .map(|zone| json_cstring(zone, "zone ID"))
            .transpose()
        {
            Ok(zone_json) => zone_json,
            Err(error) => {
                return FetchQueryResultsFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let desired_keys_json = match desired_keys
            .as_ref()
            .map(|desired_keys| json_cstring(desired_keys, "desired keys"))
            .transpose()
        {
            Ok(desired_keys_json) => desired_keys_json,
            Err(error) => {
                return FetchQueryResultsFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let Ok(results_limit) = i32::try_from(results_limit.unwrap_or_default()) else {
            return FetchQueryResultsFuture {
                inner: ready_error_future(CloudKitError::bridge(
                    -1,
                    "results limit exceeds supported i32 range",
                )),
            };
        };
        let (future, context) = AsyncCompletion::create();
        // SAFETY: all C-string arguments are valid for the duration of the call; `context` is a valid `SyncCompletionPtr` consumed exactly once by the callback.
        unsafe {
            ffi::ck_database_fetch_query_results_async(
                opt_cstring_ptr(&identifier),
                self.database_scope() as i32,
                query_json.as_ptr(),
                opt_cstring_ptr(&zone_json),
                opt_cstring_ptr(&desired_keys_json),
                results_limit,
                json_callback,
                context,
            );
        };
        FetchQueryResultsFuture { inner: future }
    }

    /// Wraps the async form of `CKDatabase.allRecordZones()`.
    pub fn fetch_all_record_zones_async(&self) -> FetchAllRecordZonesFuture {
        let identifier = match optional_cstring_from_str(
            self.container().container_identifier(),
            "container identifier",
        ) {
            Ok(identifier) => identifier,
            Err(error) => {
                return FetchAllRecordZonesFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let (future, context) = AsyncCompletion::create();
        // SAFETY: all C-string arguments are valid for the duration of the call; `context` is a valid `SyncCompletionPtr` consumed exactly once by the callback.
        unsafe {
            ffi::ck_database_fetch_all_record_zones_async(
                opt_cstring_ptr(&identifier),
                self.database_scope() as i32,
                json_callback,
                context,
            );
        };
        FetchAllRecordZonesFuture { inner: future }
    }
}

impl CKModifyRecordsOperation {
    /// Executes `CKModifyRecordsOperation` asynchronously in the given `CKDatabase`.
    pub fn execute_in_async(&self, database: &CKDatabase) -> ModifyRecordsFuture {
        let identifier = match optional_cstring_from_str(
            database.container().container_identifier(),
            "container identifier",
        ) {
            Ok(identifier) => identifier,
            Err(error) => {
                return ModifyRecordsFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let payload = CKModifyRecordsOperationPayload {
            records_to_save: self
                .records_to_save()
                .iter()
                .map(CKRecord::to_payload)
                .collect(),
            record_ids_to_delete: self
                .record_ids_to_delete()
                .iter()
                .map(CKRecordID::to_payload)
                .collect(),
            save_policy: self.save_policy() as i32,
            atomic: self.atomic(),
        };
        let operation_json = match json_cstring(&payload, "modify-records operation") {
            Ok(operation_json) => operation_json,
            Err(error) => {
                return ModifyRecordsFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let (future, context) = AsyncCompletion::create();
        // SAFETY: all C-string arguments are valid for the duration of the call; `context` is a valid `SyncCompletionPtr` consumed exactly once by the callback.
        unsafe {
            ffi::ck_database_modify_records_async(
                opt_cstring_ptr(&identifier),
                database.database_scope() as i32,
                operation_json.as_ptr(),
                json_callback,
                context,
            );
        };
        ModifyRecordsFuture { inner: future }
    }
}

impl CKFetchDatabaseChangesOperation {
    /// Executes `CKFetchDatabaseChangesOperation` asynchronously in the given `CKDatabase`.
    pub fn execute_in_async(&self, database: &CKDatabase) -> FetchDatabaseChangesFuture {
        let identifier = match optional_cstring_from_str(
            database.container().container_identifier(),
            "container identifier",
        ) {
            Ok(identifier) => identifier,
            Err(error) => {
                return FetchDatabaseChangesFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let payload = CKFetchDatabaseChangesOperationPayload {
            previous_server_change_token: self
                .previous_server_change_token()
                .map(CKServerChangeToken::to_payload),
            results_limit: self.results_limit(),
            fetch_all_changes: self.fetch_all_changes(),
        };
        let operation_json = match json_cstring(&payload, "fetch-database-changes operation") {
            Ok(operation_json) => operation_json,
            Err(error) => {
                return FetchDatabaseChangesFuture {
                    inner: ready_error_future(error),
                };
            }
        };
        let (future, context) = AsyncCompletion::create();
        // SAFETY: all C-string arguments are valid for the duration of the call; `context` is a valid `SyncCompletionPtr` consumed exactly once by the callback.
        unsafe {
            ffi::ck_database_fetch_changes_async(
                opt_cstring_ptr(&identifier),
                database.database_scope() as i32,
                operation_json.as_ptr(),
                json_callback,
                context,
            );
        };
        FetchDatabaseChangesFuture { inner: future }
    }
}
