use core::ffi::{c_char, c_void};
use core::ptr;

use crate::database::{CKDatabase, CKDatabaseScope};
use crate::error::CloudKitError;
use crate::ffi;
use crate::private::{
    box_closure, error_from_status, json_cstring, opt_cstring_ptr, optional_cstring_from_str,
    parse_borrowed_error_ptr, parse_json_ptr, parse_json_str,
};
use crate::record::CKRecordID;
use crate::share::CKShareParticipant;
use crate::user_identity::{CKUserIdentity, CKUserIdentityLookupInfo};
use doom_fish_utils::panic_safe::catch_user_panic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AccountStatus {
    CouldNotDetermine,
    Available,
    Restricted,
    NoAccount,
    TemporarilyUnavailable,
    Unknown(i32),
}

impl AccountStatus {
    pub(crate) const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::CouldNotDetermine,
            1 => Self::Available,
            2 => Self::Restricted,
            3 => Self::NoAccount,
            4 => Self::TemporarilyUnavailable,
            other => Self::Unknown(other),
        }
    }
}

impl core::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let label = match self {
            Self::CouldNotDetermine => "couldNotDetermine",
            Self::Available => "available",
            Self::Restricted => "restricted",
            Self::NoAccount => "noAccount",
            Self::TemporarilyUnavailable => "temporarilyUnavailable",
            Self::Unknown(raw) => return write!(f, "unknown({raw})"),
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKApplicationPermissionStatus {
    InitialState,
    CouldNotComplete,
    Denied,
    Granted,
    Unknown(i32),
}

impl CKApplicationPermissionStatus {
    #[cfg(feature = "async")]
    pub(crate) const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::InitialState,
            1 => Self::CouldNotComplete,
            2 => Self::Denied,
            3 => Self::Granted,
            other => Self::Unknown(other),
        }
    }
}

impl core::fmt::Display for CKApplicationPermissionStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let label = match self {
            Self::InitialState => "initialState",
            Self::CouldNotComplete => "couldNotComplete",
            Self::Denied => "denied",
            Self::Granted => "granted",
            Self::Unknown(raw) => return write!(f, "unknown({raw})"),
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CKApplicationPermissions(u64);

impl CKApplicationPermissions {
    pub const USER_DISCOVERABILITY: Self = Self(1 << 0);

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl core::ops::BitOr for CKApplicationPermissions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for CKApplicationPermissions {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKContainer {
    identifier: Option<String>,
}

impl Default for CKContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CKContainer {
    pub fn new() -> Self {
        Self { identifier: None }
    }

    pub fn default() -> Self {
        Self::new()
    }

    pub fn container(identifier: impl Into<String>) -> Self {
        Self {
            identifier: Some(identifier.into()),
        }
    }

    pub fn container_identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    pub fn private_cloud_database(&self) -> CKDatabase {
        CKDatabase::new(self.clone(), CKDatabaseScope::Private)
    }

    pub fn public_cloud_database(&self) -> CKDatabase {
        CKDatabase::new(self.clone(), CKDatabaseScope::Public)
    }

    pub fn shared_cloud_database(&self) -> CKDatabase {
        CKDatabase::new(self.clone(), CKDatabaseScope::Shared)
    }

    pub fn database_with_scope(&self, scope: CKDatabaseScope) -> CKDatabase {
        CKDatabase::new(self.clone(), scope)
    }

    pub fn account_status(&self) -> Result<AccountStatus, CloudKitError> {
        let identifier =
            optional_cstring_from_str(self.identifier.as_deref(), "container identifier")?;
        let mut out_status = 0_i32;
        let mut out_error: *mut c_char = ptr::null_mut();
        // SAFETY: all pointer arguments are either null (via `opt_cstring_ptr`) or valid null-terminated C strings whose lifetimes cover the call. Any output pointers are valid mutable pointers; owned string outputs such as `out_json` and `out_error` start as null so the bridge can allocate and transfer ownership.
        let status = unsafe {
            ffi::ck_container_account_status_sync(
                opt_cstring_ptr(&identifier),
                &mut out_status,
                &mut out_error,
            )
        };
        if status != ffi::status::OK {
            // SAFETY: `out_error` is either null or a bridge-allocated C string; `error_from_status` consumes it.
            return Err(unsafe { error_from_status(status, out_error) });
        }
        Ok(AccountStatus::from_raw(out_status))
    }

    pub fn account_status_with_completion_handler<F>(
        &self,
        callback: F,
    ) -> Result<(), CloudKitError>
    where
        F: FnOnce(Result<AccountStatus, CloudKitError>) + Send + 'static,
    {
        let identifier =
            optional_cstring_from_str(self.identifier.as_deref(), "container identifier")?;
        let callback_ptr = box_closure(Box::new(callback) as AccountStatusCallback);
        unsafe {
            ffi::ck_container_account_status_async(
                opt_cstring_ptr(&identifier),
                account_status_trampoline,
                callback_ptr,
            );
        }
        Ok(())
    }

    pub fn fetch_user_record_id(&self) -> Result<CKRecordID, CloudKitError> {
        let identifier =
            optional_cstring_from_str(self.identifier.as_deref(), "container identifier")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        // SAFETY: all pointer arguments are either null (via `opt_cstring_ptr`) or valid null-terminated C strings whose lifetimes cover the call. Any output pointers are valid mutable pointers; owned string outputs such as `out_json` and `out_error` start as null so the bridge can allocate and transfer ownership.
        let status = unsafe {
            ffi::ck_container_fetch_user_record_id_sync(
                opt_cstring_ptr(&identifier),
                &mut out_json,
                &mut out_error,
            )
        };
        if status != ffi::status::OK {
            // SAFETY: `out_error` is either null or a bridge-allocated C string; `error_from_status` consumes it.
            return Err(unsafe { error_from_status(status, out_error) });
        }
        // SAFETY: `out_json` is either null or a bridge-allocated C string; `parse_json_ptr` frees it via `ck_string_free`.
        let record_id = unsafe {
            parse_json_ptr::<crate::private::CKRecordIDPayload>(out_json, "user record ID")?
        };
        Ok(CKRecordID::from_payload(record_id))
    }

    pub fn fetch_user_record_id_with_completion_handler<F>(
        &self,
        callback: F,
    ) -> Result<(), CloudKitError>
    where
        F: FnOnce(Result<CKRecordID, CloudKitError>) + Send + 'static,
    {
        let identifier =
            optional_cstring_from_str(self.identifier.as_deref(), "container identifier")?;
        let callback_ptr = box_closure(Box::new(callback) as RecordIdCallback);
        unsafe {
            ffi::ck_container_fetch_user_record_id_async(
                opt_cstring_ptr(&identifier),
                record_id_trampoline,
                callback_ptr,
            );
        }
        Ok(())
    }

    pub fn discover_user_identity(
        &self,
        lookup_info: &CKUserIdentityLookupInfo,
    ) -> Result<CKUserIdentity, CloudKitError> {
        let identifier =
            optional_cstring_from_str(self.identifier.as_deref(), "container identifier")?;
        let lookup_json = json_cstring(&lookup_info.to_payload(), "user identity lookup info")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        // SAFETY: all pointer arguments are either null (via `opt_cstring_ptr`) or valid null-terminated C strings whose lifetimes cover the call. Any output pointers are valid mutable pointers; owned string outputs such as `out_json` and `out_error` start as null so the bridge can allocate and transfer ownership.
        let status = unsafe {
            ffi::ck_container_discover_user_identity_sync(
                opt_cstring_ptr(&identifier),
                lookup_json.as_ptr(),
                &mut out_json,
                &mut out_error,
            )
        };
        if status != ffi::status::OK {
            // SAFETY: `out_error` is either null or a bridge-allocated C string; `error_from_status` consumes it.
            return Err(unsafe { error_from_status(status, out_error) });
        }
        // SAFETY: `out_json` is either null or a bridge-allocated C string; `parse_json_ptr` frees it via `ck_string_free`.
        let payload = unsafe {
            parse_json_ptr::<crate::private::CKUserIdentityPayload>(out_json, "user identity")?
        };
        Ok(CKUserIdentity::from_payload(payload))
    }

    pub fn discover_user_identity_with_email_address(
        &self,
        email_address: impl Into<String>,
    ) -> Result<CKUserIdentity, CloudKitError> {
        self.discover_user_identity(&CKUserIdentityLookupInfo::with_email_address(email_address))
    }

    pub fn discover_user_identity_with_phone_number(
        &self,
        phone_number: impl Into<String>,
    ) -> Result<CKUserIdentity, CloudKitError> {
        self.discover_user_identity(&CKUserIdentityLookupInfo::with_phone_number(phone_number))
    }

    pub fn discover_user_identity_with_user_record_id(
        &self,
        user_record_id: CKRecordID,
    ) -> Result<CKUserIdentity, CloudKitError> {
        self.discover_user_identity(&CKUserIdentityLookupInfo::with_user_record_id(
            user_record_id,
        ))
    }

    pub fn fetch_share_participant(
        &self,
        lookup_info: &CKUserIdentityLookupInfo,
    ) -> Result<CKShareParticipant, CloudKitError> {
        let identifier =
            optional_cstring_from_str(self.identifier.as_deref(), "container identifier")?;
        let lookup_json = json_cstring(&lookup_info.to_payload(), "share participant lookup info")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        // SAFETY: all pointer arguments are either null (via `opt_cstring_ptr`) or valid null-terminated C strings whose lifetimes cover the call. Any output pointers are valid mutable pointers; owned string outputs such as `out_json` and `out_error` start as null so the bridge can allocate and transfer ownership.
        let status = unsafe {
            ffi::ck_container_fetch_share_participant_sync(
                opt_cstring_ptr(&identifier),
                lookup_json.as_ptr(),
                &mut out_json,
                &mut out_error,
            )
        };
        if status != ffi::status::OK {
            // SAFETY: `out_error` is either null or a bridge-allocated C string; `error_from_status` consumes it.
            return Err(unsafe { error_from_status(status, out_error) });
        }
        // SAFETY: `out_json` is either null or a bridge-allocated C string; `parse_json_ptr` frees it via `ck_string_free`.
        let payload = unsafe {
            parse_json_ptr::<crate::private::CKShareParticipantPayload>(
                out_json,
                "share participant",
            )?
        };
        Ok(CKShareParticipant::from_payload(payload))
    }

    pub fn fetch_share_participant_with_email_address(
        &self,
        email_address: impl Into<String>,
    ) -> Result<CKShareParticipant, CloudKitError> {
        self.fetch_share_participant(&CKUserIdentityLookupInfo::with_email_address(email_address))
    }

    pub fn fetch_share_participant_with_phone_number(
        &self,
        phone_number: impl Into<String>,
    ) -> Result<CKShareParticipant, CloudKitError> {
        self.fetch_share_participant(&CKUserIdentityLookupInfo::with_phone_number(phone_number))
    }

    pub fn fetch_share_participant_with_user_record_id(
        &self,
        user_record_id: CKRecordID,
    ) -> Result<CKShareParticipant, CloudKitError> {
        self.fetch_share_participant(&CKUserIdentityLookupInfo::with_user_record_id(
            user_record_id,
        ))
    }
}

type AccountStatusCallback = Box<dyn FnOnce(Result<AccountStatus, CloudKitError>) + Send + 'static>;
type RecordIdCallback = Box<dyn FnOnce(Result<CKRecordID, CloudKitError>) + Send + 'static>;

unsafe extern "C" fn account_status_trampoline(
    refcon: *mut c_void,
    status_raw: i32,
    error_json: *const c_char,
) {
    // SAFETY: `refcon` was set via `box_closure(Box::new(callback))` and this
    // trampoline fires exactly once.
    catch_user_panic("cloudkit::account_status_trampoline", || {
        let callback: Box<AccountStatusCallback> = unsafe { Box::from_raw(refcon.cast()) };
        let result = if error_json.is_null() {
            Ok(AccountStatus::from_raw(status_raw))
        } else {
            // SAFETY: `error_json` is non-null and points to a bridge-owned null-terminated string.
            Err(unsafe { parse_borrowed_error_ptr(error_json) })
        };
        callback(result);
    });
}

unsafe extern "C" fn record_id_trampoline(
    refcon: *mut c_void,
    json: *const c_char,
    error_json: *const c_char,
) {
    // SAFETY: `refcon` was set via `box_closure(Box::new(callback))` and this
    // trampoline fires exactly once.
    catch_user_panic("cloudkit::record_id_trampoline", || {
        let callback: Box<RecordIdCallback> = unsafe { Box::from_raw(refcon.cast()) };
        let result = if error_json.is_null() {
            let payload = parse_json_str::<crate::private::CKRecordIDPayload>(
                // SAFETY: bridge guarantees `json` is a valid null-terminated C string when
                // `error_json` is null.
                &unsafe { std::ffi::CStr::from_ptr(json) }.to_string_lossy(),
                "user record ID",
            );
            payload.map(CKRecordID::from_payload)
        } else {
            // SAFETY: `error_json` is non-null and points to a bridge-owned null-terminated string.
            Err(unsafe { parse_borrowed_error_ptr(error_json) })
        };
        callback(result);
    });
}
