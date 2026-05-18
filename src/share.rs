use core::ffi::c_char;
use core::ptr;

use crate::error::CloudKitError;
use crate::ffi;
use crate::private::{
    error_from_status, json_cstring, parse_json_ptr, CKShareParticipantPayload, CKSharePayload,
};
use crate::record::{CKRecord, CKRecordID, CKRecordZoneID};
use crate::user_identity::CKUserIdentity;

/// Mirrors `CKShareParticipantAcceptanceStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKShareParticipantAcceptanceStatus {
    /// Mirrors `CKShareParticipantAcceptanceStatus.unknown`.
    Unknown,
    /// Mirrors `CKShareParticipantAcceptanceStatus.pending`.
    Pending,
    /// Mirrors `CKShareParticipantAcceptanceStatus.accepted`.
    Accepted,
    /// Mirrors `CKShareParticipantAcceptanceStatus.removed`.
    Removed,
    /// Mirrors `CKShareParticipantAcceptanceStatus.unknownValue`.
    UnknownValue(i32),
}

impl CKShareParticipantAcceptanceStatus {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Unknown,
            1 => Self::Pending,
            2 => Self::Accepted,
            3 => Self::Removed,
            other => Self::UnknownValue(other),
        }
    }
}

/// Mirrors `CKShareParticipantPermission`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKShareParticipantPermission {
    /// Mirrors `CKShareParticipantPermission.unknown`.
    Unknown,
    /// Mirrors `CKShareParticipantPermission.none`.
    None,
    /// Mirrors `CKShareParticipantPermission.readOnly`.
    ReadOnly,
    /// Mirrors `CKShareParticipantPermission.readWrite`.
    ReadWrite,
    /// Mirrors `CKShareParticipantPermission.unknownValue`.
    UnknownValue(i32),
}

impl CKShareParticipantPermission {
    pub(crate) const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Unknown,
            1 => Self::None,
            2 => Self::ReadOnly,
            3 => Self::ReadWrite,
            other => Self::UnknownValue(other),
        }
    }

    pub(crate) const fn to_raw(self) -> i32 {
        match self {
            Self::Unknown => 0,
            Self::None => 1,
            Self::ReadOnly => 2,
            Self::ReadWrite => 3,
            Self::UnknownValue(raw) => raw,
        }
    }
}

/// Mirrors `CKShareParticipantRole`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKShareParticipantRole {
    /// Mirrors `CKShareParticipantRole.unknown`.
    Unknown,
    /// Mirrors `CKShareParticipantRole.owner`.
    Owner,
    /// Mirrors `CKShareParticipantRole.administrator`.
    Administrator,
    /// Mirrors `CKShareParticipantRole.privateUser`.
    PrivateUser,
    /// Mirrors `CKShareParticipantRole.publicUser`.
    PublicUser,
    /// Mirrors `CKShareParticipantRole.unknownValue`.
    UnknownValue(i32),
}

impl CKShareParticipantRole {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Unknown,
            1 => Self::Owner,
            2 => Self::Administrator,
            3 => Self::PrivateUser,
            4 => Self::PublicUser,
            other => Self::UnknownValue(other),
        }
    }
}

/// Wraps `CKShareParticipant`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKShareParticipant {
    archived_data: Vec<u8>,
    user_identity: CKUserIdentity,
    role: Option<CKShareParticipantRole>,
    permission: CKShareParticipantPermission,
    acceptance_status: CKShareParticipantAcceptanceStatus,
    participant_id: String,
    is_approved_requester: Option<bool>,
    date_added_to_share: Option<String>,
}

impl CKShareParticipant {
    /// Mirrors `CKShareParticipant.oneTimeURLParticipant`.
    pub fn one_time_url_participant() -> Result<Self, CloudKitError> {
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status =
            unsafe { ffi::ck_share_create_one_time_url_participant(&mut out_json, &mut out_error) };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        let payload =
            unsafe { parse_json_ptr::<CKShareParticipantPayload>(out_json, "share participant")? };
        Ok(Self::from_payload(payload))
    }

    /// Mirrors archived data stored by `CKShareParticipant`.
    pub fn archived_data(&self) -> &[u8] {
        &self.archived_data
    }

    /// Mirrors `CKShareParticipant.userIDentity`.
    pub fn user_identity(&self) -> &CKUserIdentity {
        &self.user_identity
    }

    /// Mirrors `CKShareParticipant.role`.
    pub const fn role(&self) -> Option<CKShareParticipantRole> {
        self.role
    }

    /// Mirrors `CKShareParticipant.permission`.
    pub const fn permission(&self) -> CKShareParticipantPermission {
        self.permission
    }

    /// Mirrors `CKShareParticipant.acceptanceStatus`.
    pub const fn acceptance_status(&self) -> CKShareParticipantAcceptanceStatus {
        self.acceptance_status
    }

    /// Mirrors `CKShareParticipant.participantID`.
    pub fn participant_id(&self) -> &str {
        &self.participant_id
    }

    /// Mirrors `CKShareParticipant.isApprovedRequester`.
    pub const fn is_approved_requester(&self) -> Option<bool> {
        self.is_approved_requester
    }

    /// Mirrors `CKShareParticipant.dateAddedToShare`.
    pub fn date_added_to_share(&self) -> Option<&str> {
        self.date_added_to_share.as_deref()
    }

    pub(crate) fn from_payload(payload: CKShareParticipantPayload) -> Self {
        Self {
            archived_data: payload.archived_data,
            user_identity: CKUserIdentity::from_payload(payload.user_identity),
            role: payload.role.map(CKShareParticipantRole::from_raw),
            permission: CKShareParticipantPermission::from_raw(payload.permission),
            acceptance_status: CKShareParticipantAcceptanceStatus::from_raw(
                payload.acceptance_status,
            ),
            participant_id: payload.participant_id,
            is_approved_requester: payload.is_approved_requester,
            date_added_to_share: payload.date_added_to_share,
        }
    }

    pub(crate) fn to_payload(&self) -> CKShareParticipantPayload {
        CKShareParticipantPayload {
            archived_data: self.archived_data.clone(),
            user_identity: crate::private::CKUserIdentityPayload {
                archived_data: self.user_identity.archived_data().to_vec(),
                user_record_id: self
                    .user_identity
                    .user_record_id()
                    .map(CKRecordID::to_payload),
                lookup_info: self
                    .user_identity
                    .lookup_info()
                    .map(crate::user_identity::CKUserIdentityLookupInfo::to_payload),
                name_components: self
                    .user_identity
                    .name_components()
                    .map(crate::user_identity::CKPersonNameComponents::to_payload),
                hasi_cloud_account: self.user_identity.has_i_cloud_account(),
                contact_identifiers: self.user_identity.contact_identifiers().to_vec(),
            },
            role: self.role.map(|role| match role {
                CKShareParticipantRole::Unknown => 0,
                CKShareParticipantRole::Owner => 1,
                CKShareParticipantRole::Administrator => 2,
                CKShareParticipantRole::PrivateUser => 3,
                CKShareParticipantRole::PublicUser => 4,
                CKShareParticipantRole::UnknownValue(raw) => raw,
            }),
            permission: self.permission.to_raw(),
            acceptance_status: match self.acceptance_status {
                CKShareParticipantAcceptanceStatus::Unknown => 0,
                CKShareParticipantAcceptanceStatus::Pending => 1,
                CKShareParticipantAcceptanceStatus::Accepted => 2,
                CKShareParticipantAcceptanceStatus::Removed => 3,
                CKShareParticipantAcceptanceStatus::UnknownValue(raw) => raw,
            },
            participant_id: self.participant_id.clone(),
            is_approved_requester: self.is_approved_requester,
            date_added_to_share: self.date_added_to_share.clone(),
        }
    }
}

/// Wraps `CKShare`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKShare {
    share_record: CKRecord,
    root_record: Option<CKRecord>,
    zone_id: CKRecordZoneID,
    public_permission: CKShareParticipantPermission,
    url: Option<String>,
    participants: Vec<CKShareParticipant>,
    owner: Option<CKShareParticipant>,
    current_user_participant: Option<CKShareParticipant>,
    title: Option<String>,
    thumbnail_image_data: Option<Vec<u8>>,
    share_type: Option<String>,
    allows_access_requests: Option<bool>,
    is_zone_wide: bool,
}

impl CKShare {
    /// Mirrors `CKShare.newRootRecord`.
    pub fn new_root_record(root_record: &CKRecord) -> Result<Self, CloudKitError> {
        let root_record_json = json_cstring(&root_record.to_payload(), "root record")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::ck_share_create_root_record(
                root_record_json.as_ptr(),
                &mut out_json,
                &mut out_error,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        let payload = unsafe { parse_json_ptr::<CKSharePayload>(out_json, "share")? };
        Ok(Self::from_payload(payload))
    }

    /// Mirrors `CKShare.newZoneWide`.
    pub fn new_zone_wide(zone_id: CKRecordZoneID) -> Result<Self, CloudKitError> {
        let zone_json = json_cstring(&zone_id.to_payload(), "zone ID")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::ck_share_create_zone_wide(zone_json.as_ptr(), &mut out_json, &mut out_error)
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        let payload = unsafe { parse_json_ptr::<CKSharePayload>(out_json, "share")? };
        Ok(Self::from_payload(payload))
    }

    /// Mirrors `CKShare.shareRecord`.
    pub fn share_record(&self) -> &CKRecord {
        &self.share_record
    }

    /// Mirrors `CKShare.rootRecord`.
    pub const fn root_record(&self) -> Option<&CKRecord> {
        self.root_record.as_ref()
    }

    /// Mirrors `CKShare.zoneID`.
    pub fn zone_id(&self) -> &CKRecordZoneID {
        &self.zone_id
    }

    /// Mirrors `CKShare.publicPermission`.
    pub const fn public_permission(&self) -> CKShareParticipantPermission {
        self.public_permission
    }

    /// Mirrors `CKShare.url`.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Mirrors `CKShare.participants`.
    pub fn participants(&self) -> &[CKShareParticipant] {
        &self.participants
    }

    /// Mirrors `CKShare.owner`.
    pub const fn owner(&self) -> Option<&CKShareParticipant> {
        self.owner.as_ref()
    }

    /// Mirrors `CKShare.currentUserParticipant`.
    pub const fn current_user_participant(&self) -> Option<&CKShareParticipant> {
        self.current_user_participant.as_ref()
    }

    /// Mirrors `CKShare.title`.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Mirrors `CKShare.thumbnailImageData`.
    pub fn thumbnail_image_data(&self) -> Option<&[u8]> {
        self.thumbnail_image_data.as_deref()
    }

    /// Mirrors `CKShare.shareType`.
    pub fn share_type(&self) -> Option<&str> {
        self.share_type.as_deref()
    }

    /// Mirrors `CKShare.allowsAccessRequests`.
    pub const fn allows_access_requests(&self) -> Option<bool> {
        self.allows_access_requests
    }

    /// Mirrors `CKShare.isZoneWide`.
    pub const fn is_zone_wide(&self) -> bool {
        self.is_zone_wide
    }

    /// Mirrors `CKShare.asRecord`.
    pub fn as_record(&self) -> CKRecord {
        self.share_record.clone()
    }

    /// Sets the value mirroring `CKShare.publicPermission`.
    pub fn with_public_permission(
        mut self,
        public_permission: CKShareParticipantPermission,
    ) -> Result<Self, CloudKitError> {
        self.public_permission = public_permission;
        self.normalize()
    }

    /// Sets the value mirroring `CKShare.title`.
    pub fn with_title(mut self, title: impl Into<String>) -> Result<Self, CloudKitError> {
        self.title = Some(title.into());
        self.normalize()
    }

    /// Sets the value mirroring `CKShare.thumbnailImageData`.
    pub fn with_thumbnail_image_data(
        mut self,
        thumbnail_image_data: Vec<u8>,
    ) -> Result<Self, CloudKitError> {
        self.thumbnail_image_data = Some(thumbnail_image_data);
        self.normalize()
    }

    /// Sets the value mirroring `CKShare.shareType`.
    pub fn with_share_type(mut self, share_type: impl Into<String>) -> Result<Self, CloudKitError> {
        self.share_type = Some(share_type.into());
        self.normalize()
    }

    /// Sets the value mirroring `CKShare.allowsAccessRequests`.
    pub fn with_allows_access_requests(
        mut self,
        allows_access_requests: bool,
    ) -> Result<Self, CloudKitError> {
        self.allows_access_requests = Some(allows_access_requests);
        self.normalize()
    }

    /// Mirrors `CKShare.addParticipant`.
    pub fn add_participant(
        mut self,
        participant: CKShareParticipant,
    ) -> Result<Self, CloudKitError> {
        self.participants.push(participant);
        self.normalize()
    }

    /// Mirrors `CKShare.removeParticipant`.
    pub fn remove_participant(mut self, participant_id: &str) -> Result<Self, CloudKitError> {
        self.participants
            .retain(|participant| participant.participant_id() != participant_id);
        self.normalize()
    }

    fn normalize(self) -> Result<Self, CloudKitError> {
        let share_json = json_cstring(&self.to_payload(), "share")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status =
            unsafe { ffi::ck_share_normalize(share_json.as_ptr(), &mut out_json, &mut out_error) };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        let payload = unsafe { parse_json_ptr::<CKSharePayload>(out_json, "share")? };
        Ok(Self::from_payload(payload))
    }

    pub(crate) fn from_payload(payload: CKSharePayload) -> Self {
        Self {
            share_record: CKRecord::from_payload(payload.share_record),
            root_record: payload.root_record.map(CKRecord::from_payload),
            zone_id: CKRecordZoneID::from_payload(payload.zone_id),
            public_permission: CKShareParticipantPermission::from_raw(payload.public_permission),
            url: payload.url,
            participants: payload
                .participants
                .into_iter()
                .map(CKShareParticipant::from_payload)
                .collect(),
            owner: payload.owner.map(CKShareParticipant::from_payload),
            current_user_participant: payload
                .current_user_participant
                .map(CKShareParticipant::from_payload),
            title: payload.title,
            thumbnail_image_data: payload.thumbnail_image_data,
            share_type: payload.share_type,
            allows_access_requests: payload.allows_access_requests,
            is_zone_wide: payload.is_zone_wide,
        }
    }

    pub(crate) fn to_payload(&self) -> CKSharePayload {
        CKSharePayload {
            share_record: self.share_record.to_payload(),
            root_record: self.root_record.as_ref().map(CKRecord::to_payload),
            zone_id: self.zone_id.to_payload(),
            public_permission: self.public_permission.to_raw(),
            url: self.url.clone(),
            participants: self
                .participants
                .iter()
                .map(CKShareParticipant::to_payload)
                .collect(),
            owner: self.owner.as_ref().map(CKShareParticipant::to_payload),
            current_user_participant: self
                .current_user_participant
                .as_ref()
                .map(CKShareParticipant::to_payload),
            title: self.title.clone(),
            thumbnail_image_data: self.thumbnail_image_data.clone(),
            share_type: self.share_type.clone(),
            allows_access_requests: self.allows_access_requests,
            is_zone_wide: self.is_zone_wide,
        }
    }
}

/// Mirrors `CKShareParticipantType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKShareParticipantType {
    /// Mirrors `CKShareParticipantType.unknown`.
    Unknown,
    /// Mirrors `CKShareParticipantType.owner`.
    Owner,
    /// Mirrors `CKShareParticipantType.privateUser`.
    PrivateUser,
    /// Mirrors `CKShareParticipantType.publicUser`.
    PublicUser,
    /// Mirrors `CKShareParticipantType.unknownValue`.
    UnknownValue(i32),
}

impl CKShareParticipantType {
    pub(crate) const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Unknown,
            1 => Self::Owner,
            3 => Self::PrivateUser,
            4 => Self::PublicUser,
            other => Self::UnknownValue(other),
        }
    }
}

impl CKShareParticipant {
    /// Mirrors `CKShareParticipant.participantType`.
    pub const fn participant_type(&self) -> CKShareParticipantType {
        match self.role {
            Some(CKShareParticipantRole::Owner) => CKShareParticipantType::Owner,
            Some(CKShareParticipantRole::PrivateUser) => CKShareParticipantType::PrivateUser,
            Some(CKShareParticipantRole::PublicUser) => CKShareParticipantType::PublicUser,
            Some(CKShareParticipantRole::UnknownValue(raw)) => {
                CKShareParticipantType::UnknownValue(raw)
            }
            _ => CKShareParticipantType::Unknown,
        }
    }
}

/// Wraps `CKSharingParticipantAccessOption`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CKSharingParticipantAccessOption(u64);

impl CKSharingParticipantAccessOption {
    /// Mirrors the `anyoneWithLink` option on `CKSharingParticipantAccessOption`.
    pub const ANYONE_WITH_LINK: Self = Self(1 << 0);
    /// Mirrors the `specifiedRecipientsOnly` option on `CKSharingParticipantAccessOption`.
    pub const SPECIFIED_RECIPIENTS_ONLY: Self = Self(1 << 1);
    /// Mirrors the `any` option on `CKSharingParticipantAccessOption`.
    pub const ANY: Self = Self(Self::ANYONE_WITH_LINK.0 | Self::SPECIFIED_RECIPIENTS_ONLY.0);

    /// Mirrors `CKSharingParticipantAccessOption.rawValue`.
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Mirrors `CKSharingParticipantAccessOption.contains(_:)`.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl core::ops::BitOr for CKSharingParticipantAccessOption {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for CKSharingParticipantAccessOption {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Wraps `CKSharingParticipantPermissionOption`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CKSharingParticipantPermissionOption(u64);

impl CKSharingParticipantPermissionOption {
    /// Mirrors the `readOnly` option on `CKSharingParticipantPermissionOption`.
    pub const READ_ONLY: Self = Self(1 << 0);
    /// Mirrors the `readWrite` option on `CKSharingParticipantPermissionOption`.
    pub const READ_WRITE: Self = Self(1 << 1);
    /// Mirrors the `any` option on `CKSharingParticipantPermissionOption`.
    pub const ANY: Self = Self(Self::READ_ONLY.0 | Self::READ_WRITE.0);

    /// Mirrors `CKSharingParticipantPermissionOption.rawValue`.
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Mirrors `CKSharingParticipantPermissionOption.contains(_:)`.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl core::ops::BitOr for CKSharingParticipantPermissionOption {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for CKSharingParticipantPermissionOption {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Wraps `CKAllowedSharingOptions`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKAllowedSharingOptions {
    allowed_participant_permission_options: CKSharingParticipantPermissionOption,
    allowed_participant_access_options: CKSharingParticipantAccessOption,
}

impl CKAllowedSharingOptions {
    /// Creates a wrapper mirroring `CKAllowedSharingOptions`.
    pub fn new(
        allowed_participant_permission_options: CKSharingParticipantPermissionOption,
        allowed_participant_access_options: CKSharingParticipantAccessOption,
    ) -> Self {
        Self {
            allowed_participant_permission_options,
            allowed_participant_access_options,
        }
    }

    /// Mirrors `CKAllowedSharingOptions.standard`.
    pub fn standard() -> Self {
        Self::new(
            CKSharingParticipantPermissionOption::ANY,
            CKSharingParticipantAccessOption::ANY,
        )
    }

    /// Mirrors `CKAllowedSharingOptions.allowedParticipantPermissionOptions`.
    pub const fn allowed_participant_permission_options(
        &self,
    ) -> CKSharingParticipantPermissionOption {
        self.allowed_participant_permission_options
    }

    /// Mirrors `CKAllowedSharingOptions.allowedParticipantAccessOptions`.
    pub const fn allowed_participant_access_options(&self) -> CKSharingParticipantAccessOption {
        self.allowed_participant_access_options
    }
}

impl Default for CKAllowedSharingOptions {
    fn default() -> Self {
        Self::standard()
    }
}

/// Wraps `CKShareAccessRequester`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKShareAccessRequester {
    user_identity: CKUserIdentity,
    participant_lookup_info: crate::user_identity::CKUserIdentityLookupInfo,
    contact_display_name: Option<String>,
}

impl CKShareAccessRequester {
    /// Creates a wrapper mirroring `CKShareAccessRequester`.
    pub fn new(
        user_identity: CKUserIdentity,
        participant_lookup_info: crate::user_identity::CKUserIdentityLookupInfo,
    ) -> Self {
        Self {
            user_identity,
            participant_lookup_info,
            contact_display_name: None,
        }
    }

    /// Mirrors `CKShareAccessRequester.userIDentity`.
    pub const fn user_identity(&self) -> &CKUserIdentity {
        &self.user_identity
    }

    /// Mirrors `CKShareAccessRequester.participantLookupInfo`.
    pub const fn participant_lookup_info(&self) -> &crate::user_identity::CKUserIdentityLookupInfo {
        &self.participant_lookup_info
    }

    /// Mirrors `CKShareAccessRequester.contactDisplayName`.
    pub fn contact_display_name(&self) -> Option<&str> {
        self.contact_display_name.as_deref()
    }

    /// Sets the value mirroring `CKShareAccessRequester.contactDisplayName`.
    pub fn with_contact_display_name(mut self, contact_display_name: impl Into<String>) -> Self {
        self.contact_display_name = Some(contact_display_name.into());
        self
    }
}

/// Wraps `CKShareBlockedIdentity`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKShareBlockedIdentity {
    user_identity: CKUserIdentity,
    contact_display_name: Option<String>,
}

impl CKShareBlockedIdentity {
    /// Creates a wrapper mirroring `CKShareBlockedIdentity`.
    pub fn new(user_identity: CKUserIdentity) -> Self {
        Self {
            user_identity,
            contact_display_name: None,
        }
    }

    /// Mirrors `CKShareBlockedIdentity.userIDentity`.
    pub const fn user_identity(&self) -> &CKUserIdentity {
        &self.user_identity
    }

    /// Mirrors `CKShareBlockedIdentity.contactDisplayName`.
    pub fn contact_display_name(&self) -> Option<&str> {
        self.contact_display_name.as_deref()
    }

    /// Sets the value mirroring `CKShareBlockedIdentity.contactDisplayName`.
    pub fn with_contact_display_name(mut self, contact_display_name: impl Into<String>) -> Self {
        self.contact_display_name = Some(contact_display_name.into());
        self
    }
}

/// Wraps `CKShareMetadata`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKShareMetadata {
    archived_data: Vec<u8>,
    container_identifier: String,
    share: CKShare,
    hierarchical_root_record_id: Option<CKRecordID>,
    participant_role: Option<CKShareParticipantRole>,
    participant_status: CKShareParticipantAcceptanceStatus,
    participant_permission: CKShareParticipantPermission,
    owner_identity: CKUserIdentity,
    root_record: Option<CKRecord>,
    participant_type: Option<CKShareParticipantType>,
    root_record_id: Option<CKRecordID>,
}

impl CKShareMetadata {
    /// Creates a wrapper mirroring `CKShareMetadata`.
    pub fn new(
        container_identifier: impl Into<String>,
        share: CKShare,
        owner_identity: CKUserIdentity,
    ) -> Self {
        let root_record = share.root_record().cloned();
        let root_record_id = root_record
            .as_ref()
            .map(|record| record.record_id().clone());
        Self {
            archived_data: Vec::new(),
            container_identifier: container_identifier.into(),
            share,
            hierarchical_root_record_id: root_record_id.clone(),
            participant_role: None,
            participant_status: CKShareParticipantAcceptanceStatus::Unknown,
            participant_permission: CKShareParticipantPermission::None,
            owner_identity,
            root_record,
            participant_type: None,
            root_record_id,
        }
    }

    /// Mirrors archived data stored by `CKShareMetadata`.
    pub fn archived_data(&self) -> &[u8] {
        &self.archived_data
    }

    /// Mirrors `CKShareMetadata.containerIdentifier`.
    pub fn container_identifier(&self) -> &str {
        &self.container_identifier
    }

    /// Mirrors `CKShareMetadata.share`.
    pub const fn share(&self) -> &CKShare {
        &self.share
    }

    /// Mirrors `CKShareMetadata.hierarchicalRootRecordID`.
    pub const fn hierarchical_root_record_id(&self) -> Option<&CKRecordID> {
        self.hierarchical_root_record_id.as_ref()
    }

    /// Mirrors `CKShareMetadata.participantRole`.
    pub const fn participant_role(&self) -> Option<CKShareParticipantRole> {
        self.participant_role
    }

    /// Mirrors `CKShareMetadata.participantStatus`.
    pub const fn participant_status(&self) -> CKShareParticipantAcceptanceStatus {
        self.participant_status
    }

    /// Mirrors `CKShareMetadata.participantPermission`.
    pub const fn participant_permission(&self) -> CKShareParticipantPermission {
        self.participant_permission
    }

    /// Mirrors `CKShareMetadata.ownerIDentity`.
    pub const fn owner_identity(&self) -> &CKUserIdentity {
        &self.owner_identity
    }

    /// Mirrors `CKShareMetadata.rootRecord`.
    pub const fn root_record(&self) -> Option<&CKRecord> {
        self.root_record.as_ref()
    }

    /// Mirrors `CKShareMetadata.participantType`.
    pub const fn participant_type(&self) -> Option<CKShareParticipantType> {
        self.participant_type
    }

    /// Mirrors `CKShareMetadata.rootRecordID`.
    pub const fn root_record_id(&self) -> Option<&CKRecordID> {
        self.root_record_id.as_ref()
    }

    /// Sets the value mirroring `CKShareMetadata.archivedData`.
    pub fn with_archived_data(mut self, archived_data: Vec<u8>) -> Self {
        self.archived_data = archived_data;
        self
    }

    /// Sets the value mirroring `CKShareMetadata.hierarchicalRootRecordID`.
    pub fn with_hierarchical_root_record_id(
        mut self,
        hierarchical_root_record_id: CKRecordID,
    ) -> Self {
        self.hierarchical_root_record_id = Some(hierarchical_root_record_id);
        self
    }

    /// Sets the value mirroring `CKShareMetadata.participantRole`.
    pub fn with_participant_role(mut self, participant_role: CKShareParticipantRole) -> Self {
        self.participant_role = Some(participant_role);
        self
    }

    /// Sets the value mirroring `CKShareMetadata.participantStatus`.
    pub fn with_participant_status(
        mut self,
        participant_status: CKShareParticipantAcceptanceStatus,
    ) -> Self {
        self.participant_status = participant_status;
        self
    }

    /// Sets the value mirroring `CKShareMetadata.participantPermission`.
    pub fn with_participant_permission(
        mut self,
        participant_permission: CKShareParticipantPermission,
    ) -> Self {
        self.participant_permission = participant_permission;
        self
    }

    /// Sets the value mirroring `CKShareMetadata.rootRecord`.
    pub fn with_root_record(mut self, root_record: CKRecord) -> Self {
        self.root_record_id = Some(root_record.record_id().clone());
        self.root_record = Some(root_record);
        self
    }

    /// Sets the value mirroring `CKShareMetadata.participantType`.
    pub fn with_participant_type(mut self, participant_type: CKShareParticipantType) -> Self {
        self.participant_type = Some(participant_type);
        self
    }

    /// Sets the value mirroring `CKShareMetadata.rootRecordID`.
    pub fn with_root_record_id(mut self, root_record_id: CKRecordID) -> Self {
        self.root_record_id = Some(root_record_id);
        self
    }

    pub(crate) fn from_payload(payload: crate::private::CKShareMetadataPayload) -> Self {
        Self {
            archived_data: payload.archived_data,
            container_identifier: payload.container_identifier,
            share: CKShare::from_payload(payload.share),
            hierarchical_root_record_id: payload
                .hierarchical_root_record_id
                .map(CKRecordID::from_payload),
            participant_role: payload
                .participant_role
                .map(CKShareParticipantRole::from_raw),
            participant_status: CKShareParticipantAcceptanceStatus::from_raw(
                payload.participant_status,
            ),
            participant_permission: CKShareParticipantPermission::from_raw(
                payload.participant_permission,
            ),
            owner_identity: CKUserIdentity::from_payload(payload.owner_identity),
            root_record: payload.root_record.map(CKRecord::from_payload),
            participant_type: payload
                .participant_type
                .map(CKShareParticipantType::from_raw),
            root_record_id: payload.root_record_id.map(CKRecordID::from_payload),
        }
    }

    pub(crate) fn to_payload(&self) -> crate::private::CKShareMetadataPayload {
        crate::private::CKShareMetadataPayload {
            archived_data: self.archived_data.clone(),
            container_identifier: self.container_identifier.clone(),
            share: self.share.to_payload(),
            hierarchical_root_record_id: self
                .hierarchical_root_record_id
                .as_ref()
                .map(CKRecordID::to_payload),
            participant_role: self.participant_role.map(|role| match role {
                CKShareParticipantRole::Unknown => 0,
                CKShareParticipantRole::Owner => 1,
                CKShareParticipantRole::Administrator => 2,
                CKShareParticipantRole::PrivateUser => 3,
                CKShareParticipantRole::PublicUser => 4,
                CKShareParticipantRole::UnknownValue(raw) => raw,
            }),
            participant_status: match self.participant_status {
                CKShareParticipantAcceptanceStatus::Unknown => 0,
                CKShareParticipantAcceptanceStatus::Pending => 1,
                CKShareParticipantAcceptanceStatus::Accepted => 2,
                CKShareParticipantAcceptanceStatus::Removed => 3,
                CKShareParticipantAcceptanceStatus::UnknownValue(raw) => raw,
            },
            participant_permission: self.participant_permission.to_raw(),
            owner_identity: crate::private::CKUserIdentityPayload {
                archived_data: self.owner_identity.archived_data().to_vec(),
                user_record_id: self
                    .owner_identity
                    .user_record_id()
                    .map(CKRecordID::to_payload),
                lookup_info: self
                    .owner_identity
                    .lookup_info()
                    .map(crate::user_identity::CKUserIdentityLookupInfo::to_payload),
                name_components: self
                    .owner_identity
                    .name_components()
                    .map(crate::user_identity::CKPersonNameComponents::to_payload),
                hasi_cloud_account: self.owner_identity.has_i_cloud_account(),
                contact_identifiers: self.owner_identity.contact_identifiers().to_vec(),
            },
            root_record: self.root_record.as_ref().map(CKRecord::to_payload),
            participant_type: self.participant_type.map(
                |participant_type| match participant_type {
                    CKShareParticipantType::Unknown => 0,
                    CKShareParticipantType::Owner => 1,
                    CKShareParticipantType::PrivateUser => 3,
                    CKShareParticipantType::PublicUser => 4,
                    CKShareParticipantType::UnknownValue(raw) => raw,
                },
            ),
            root_record_id: self.root_record_id.as_ref().map(CKRecordID::to_payload),
        }
    }
}

/// Mirrors the save-share callback used by `CKSystemSharingUIObserver`.
pub type CKSystemSharingUIDidSaveShareHandler = dyn Fn(&CKRecordID, Option<&CKShare>, Option<&crate::error::CloudKitError>)
    + Send
    + Sync
    + 'static;
/// Mirrors the stop-sharing callback used by `CKSystemSharingUIObserver`.
pub type CKSystemSharingUIDidStopSharingHandler =
    dyn Fn(&CKRecordID, Option<&crate::error::CloudKitError>) + Send + Sync + 'static;

/// Wraps `CKSystemSharingUIObserver`.
#[derive(Clone)]
pub struct CKSystemSharingUIObserver {
    container: crate::container::CKContainer,
    did_save_share_handler: Option<std::sync::Arc<CKSystemSharingUIDidSaveShareHandler>>,
    did_stop_sharing_handler: Option<std::sync::Arc<CKSystemSharingUIDidStopSharingHandler>>,
}

impl core::fmt::Debug for CKSystemSharingUIObserver {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CKSystemSharingUIObserver")
            .field("container", &self.container)
            .finish_non_exhaustive()
    }
}

impl CKSystemSharingUIObserver {
    /// Creates a wrapper mirroring `CKSystemSharingUIObserver`.
    pub fn new(container: crate::container::CKContainer) -> Self {
        Self {
            container,
            did_save_share_handler: None,
            did_stop_sharing_handler: None,
        }
    }

    /// Mirrors `CKSystemSharingUIObserver.container`.
    pub const fn container(&self) -> &crate::container::CKContainer {
        &self.container
    }

    /// Sets the value mirroring `CKSystemSharingUIObserver.didSaveShareHandler`.
    pub fn with_did_save_share_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&CKRecordID, Option<&CKShare>, Option<&crate::error::CloudKitError>)
            + Send
            + Sync
            + 'static,
    {
        self.did_save_share_handler = Some(std::sync::Arc::new(handler));
        self
    }

    /// Sets the value mirroring `CKSystemSharingUIObserver.didStopSharingHandler`.
    pub fn with_did_stop_sharing_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&CKRecordID, Option<&crate::error::CloudKitError>) + Send + Sync + 'static,
    {
        self.did_stop_sharing_handler = Some(std::sync::Arc::new(handler));
        self
    }

    /// Mirrors `CKSystemSharingUIObserver.notifyDidSaveShare`.
    pub fn notify_did_save_share(
        &self,
        record_id: &CKRecordID,
        share: Option<&CKShare>,
        error: Option<&crate::error::CloudKitError>,
    ) {
        if let Some(handler) = &self.did_save_share_handler {
            handler(record_id, share, error);
        }
    }

    /// Mirrors `CKSystemSharingUIObserver.notifyDidStopSharing`.
    pub fn notify_did_stop_sharing(
        &self,
        record_id: &CKRecordID,
        error: Option<&crate::error::CloudKitError>,
    ) {
        if let Some(handler) = &self.did_stop_sharing_handler {
            handler(record_id, error);
        }
    }
}
