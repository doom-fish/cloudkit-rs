use core::ffi::c_char;
use core::ptr;

use crate::error::CloudKitError;
use crate::ffi;
use crate::private::{error_from_status, json_cstring, parse_json_ptr, CKShareParticipantPayload, CKSharePayload};
use crate::record::{CKRecord, CKRecordID, CKRecordZoneID};
use crate::user_identity::CKUserIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKShareParticipantAcceptanceStatus {
    Unknown,
    Pending,
    Accepted,
    Removed,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKShareParticipantPermission {
    Unknown,
    None,
    ReadOnly,
    ReadWrite,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKShareParticipantRole {
    Unknown,
    Owner,
    Administrator,
    PrivateUser,
    PublicUser,
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
    pub fn one_time_url_participant() -> Result<Self, CloudKitError> {
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::ck_share_create_one_time_url_participant(&mut out_json, &mut out_error) };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        let payload = unsafe { parse_json_ptr::<CKShareParticipantPayload>(out_json, "share participant")? };
        Ok(Self::from_payload(payload))
    }

    pub fn archived_data(&self) -> &[u8] {
        &self.archived_data
    }

    pub fn user_identity(&self) -> &CKUserIdentity {
        &self.user_identity
    }

    pub const fn role(&self) -> Option<CKShareParticipantRole> {
        self.role
    }

    pub const fn permission(&self) -> CKShareParticipantPermission {
        self.permission
    }

    pub const fn acceptance_status(&self) -> CKShareParticipantAcceptanceStatus {
        self.acceptance_status
    }

    pub fn participant_id(&self) -> &str {
        &self.participant_id
    }

    pub const fn is_approved_requester(&self) -> Option<bool> {
        self.is_approved_requester
    }

    pub fn date_added_to_share(&self) -> Option<&str> {
        self.date_added_to_share.as_deref()
    }

    pub(crate) fn from_payload(payload: CKShareParticipantPayload) -> Self {
        Self {
            archived_data: payload.archived_data,
            user_identity: CKUserIdentity::from_payload(payload.user_identity),
            role: payload.role.map(CKShareParticipantRole::from_raw),
            permission: CKShareParticipantPermission::from_raw(payload.permission),
            acceptance_status: CKShareParticipantAcceptanceStatus::from_raw(payload.acceptance_status),
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
                user_record_id: self.user_identity.user_record_id().map(CKRecordID::to_payload),
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
    pub fn new_root_record(root_record: &CKRecord) -> Result<Self, CloudKitError> {
        let root_record_json = json_cstring(&root_record.to_payload(), "root record")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::ck_share_create_root_record(root_record_json.as_ptr(), &mut out_json, &mut out_error)
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        let payload = unsafe { parse_json_ptr::<CKSharePayload>(out_json, "share")? };
        Ok(Self::from_payload(payload))
    }

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

    pub fn share_record(&self) -> &CKRecord {
        &self.share_record
    }

    pub const fn root_record(&self) -> Option<&CKRecord> {
        self.root_record.as_ref()
    }

    pub fn zone_id(&self) -> &CKRecordZoneID {
        &self.zone_id
    }

    pub const fn public_permission(&self) -> CKShareParticipantPermission {
        self.public_permission
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn participants(&self) -> &[CKShareParticipant] {
        &self.participants
    }

    pub const fn owner(&self) -> Option<&CKShareParticipant> {
        self.owner.as_ref()
    }

    pub const fn current_user_participant(&self) -> Option<&CKShareParticipant> {
        self.current_user_participant.as_ref()
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn thumbnail_image_data(&self) -> Option<&[u8]> {
        self.thumbnail_image_data.as_deref()
    }

    pub fn share_type(&self) -> Option<&str> {
        self.share_type.as_deref()
    }

    pub const fn allows_access_requests(&self) -> Option<bool> {
        self.allows_access_requests
    }

    pub const fn is_zone_wide(&self) -> bool {
        self.is_zone_wide
    }

    pub fn as_record(&self) -> CKRecord {
        self.share_record.clone()
    }

    pub fn with_public_permission(mut self, public_permission: CKShareParticipantPermission) -> Result<Self, CloudKitError> {
        self.public_permission = public_permission;
        self.normalize()
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Result<Self, CloudKitError> {
        self.title = Some(title.into());
        self.normalize()
    }

    pub fn with_thumbnail_image_data(mut self, thumbnail_image_data: Vec<u8>) -> Result<Self, CloudKitError> {
        self.thumbnail_image_data = Some(thumbnail_image_data);
        self.normalize()
    }

    pub fn with_share_type(mut self, share_type: impl Into<String>) -> Result<Self, CloudKitError> {
        self.share_type = Some(share_type.into());
        self.normalize()
    }

    pub fn with_allows_access_requests(mut self, allows_access_requests: bool) -> Result<Self, CloudKitError> {
        self.allows_access_requests = Some(allows_access_requests);
        self.normalize()
    }

    pub fn add_participant(mut self, participant: CKShareParticipant) -> Result<Self, CloudKitError> {
        self.participants.push(participant);
        self.normalize()
    }

    pub fn remove_participant(mut self, participant_id: &str) -> Result<Self, CloudKitError> {
        self.participants.retain(|participant| participant.participant_id() != participant_id);
        self.normalize()
    }

    fn normalize(self) -> Result<Self, CloudKitError> {
        let share_json = json_cstring(&self.to_payload(), "share")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::ck_share_normalize(share_json.as_ptr(), &mut out_json, &mut out_error) };
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
            participants: self.participants.iter().map(CKShareParticipant::to_payload).collect(),
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
