use crate::private::{
    CKPersonNameComponentsPayload, CKUserIdentityLookupInfoPayload, CKUserIdentityPayload,
};
use crate::record::CKRecordID;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct CKPersonNameComponents {
    name_prefix: Option<String>,
    given_name: Option<String>,
    middle_name: Option<String>,
    family_name: Option<String>,
    name_suffix: Option<String>,
    nickname: Option<String>,
}

impl CKPersonNameComponents {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name_prefix(&self) -> Option<&str> {
        self.name_prefix.as_deref()
    }

    pub fn given_name(&self) -> Option<&str> {
        self.given_name.as_deref()
    }

    pub fn middle_name(&self) -> Option<&str> {
        self.middle_name.as_deref()
    }

    pub fn family_name(&self) -> Option<&str> {
        self.family_name.as_deref()
    }

    pub fn name_suffix(&self) -> Option<&str> {
        self.name_suffix.as_deref()
    }

    pub fn nickname(&self) -> Option<&str> {
        self.nickname.as_deref()
    }

    pub fn with_name_prefix(mut self, value: impl Into<String>) -> Self {
        self.name_prefix = Some(value.into());
        self
    }

    pub fn with_given_name(mut self, value: impl Into<String>) -> Self {
        self.given_name = Some(value.into());
        self
    }

    pub fn with_middle_name(mut self, value: impl Into<String>) -> Self {
        self.middle_name = Some(value.into());
        self
    }

    pub fn with_family_name(mut self, value: impl Into<String>) -> Self {
        self.family_name = Some(value.into());
        self
    }

    pub fn with_name_suffix(mut self, value: impl Into<String>) -> Self {
        self.name_suffix = Some(value.into());
        self
    }

    pub fn with_nickname(mut self, value: impl Into<String>) -> Self {
        self.nickname = Some(value.into());
        self
    }

    pub(crate) fn from_payload(payload: CKPersonNameComponentsPayload) -> Self {
        Self {
            name_prefix: payload.name_prefix,
            given_name: payload.given_name,
            middle_name: payload.middle_name,
            family_name: payload.family_name,
            name_suffix: payload.name_suffix,
            nickname: payload.nickname,
        }
    }

    pub(crate) fn to_payload(&self) -> CKPersonNameComponentsPayload {
        CKPersonNameComponentsPayload {
            name_prefix: self.name_prefix.clone(),
            given_name: self.given_name.clone(),
            middle_name: self.middle_name.clone(),
            family_name: self.family_name.clone(),
            name_suffix: self.name_suffix.clone(),
            nickname: self.nickname.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKUserIdentityLookupInfo {
    email_address: Option<String>,
    phone_number: Option<String>,
    user_record_id: Option<CKRecordID>,
}

impl CKUserIdentityLookupInfo {
    pub fn with_email_address(email_address: impl Into<String>) -> Self {
        Self {
            email_address: Some(email_address.into()),
            phone_number: None,
            user_record_id: None,
        }
    }

    pub fn with_phone_number(phone_number: impl Into<String>) -> Self {
        Self {
            email_address: None,
            phone_number: Some(phone_number.into()),
            user_record_id: None,
        }
    }

    pub fn with_user_record_id(user_record_id: CKRecordID) -> Self {
        Self {
            email_address: None,
            phone_number: None,
            user_record_id: Some(user_record_id),
        }
    }

    pub fn email_address(&self) -> Option<&str> {
        self.email_address.as_deref()
    }

    pub fn phone_number(&self) -> Option<&str> {
        self.phone_number.as_deref()
    }

    pub const fn user_record_id(&self) -> Option<&CKRecordID> {
        self.user_record_id.as_ref()
    }

    pub(crate) fn from_payload(payload: CKUserIdentityLookupInfoPayload) -> Self {
        Self {
            email_address: payload.email_address,
            phone_number: payload.phone_number,
            user_record_id: payload.user_record_id.map(CKRecordID::from_payload),
        }
    }

    pub(crate) fn to_payload(&self) -> CKUserIdentityLookupInfoPayload {
        CKUserIdentityLookupInfoPayload {
            email_address: self.email_address.clone(),
            phone_number: self.phone_number.clone(),
            user_record_id: self.user_record_id.as_ref().map(CKRecordID::to_payload),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CKUserIdentity {
    archived_data: Vec<u8>,
    user_record_id: Option<CKRecordID>,
    lookup_info: Option<CKUserIdentityLookupInfo>,
    name_components: Option<CKPersonNameComponents>,
    hasi_cloud_account: bool,
    contact_identifiers: Vec<String>,
}

impl CKUserIdentity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn archived_data(&self) -> &[u8] {
        &self.archived_data
    }

    pub fn with_archived_data(mut self, archived_data: Vec<u8>) -> Self {
        self.archived_data = archived_data;
        self
    }

    pub fn with_user_record_id(mut self, user_record_id: CKRecordID) -> Self {
        self.user_record_id = Some(user_record_id);
        self
    }

    pub fn with_lookup_info(mut self, lookup_info: CKUserIdentityLookupInfo) -> Self {
        self.lookup_info = Some(lookup_info);
        self
    }

    pub fn with_name_components(mut self, name_components: CKPersonNameComponents) -> Self {
        self.name_components = Some(name_components);
        self
    }

    pub fn with_has_i_cloud_account(mut self, has_i_cloud_account: bool) -> Self {
        self.hasi_cloud_account = has_i_cloud_account;
        self
    }

    pub fn with_contact_identifiers(mut self, contact_identifiers: Vec<String>) -> Self {
        self.contact_identifiers = contact_identifiers;
        self
    }

    pub const fn user_record_id(&self) -> Option<&CKRecordID> {
        self.user_record_id.as_ref()
    }

    pub const fn lookup_info(&self) -> Option<&CKUserIdentityLookupInfo> {
        self.lookup_info.as_ref()
    }

    pub const fn name_components(&self) -> Option<&CKPersonNameComponents> {
        self.name_components.as_ref()
    }

    pub const fn has_i_cloud_account(&self) -> bool {
        self.hasi_cloud_account
    }

    pub fn contact_identifiers(&self) -> &[String] {
        &self.contact_identifiers
    }

    pub(crate) fn from_payload(payload: CKUserIdentityPayload) -> Self {
        Self {
            archived_data: payload.archived_data,
            user_record_id: payload.user_record_id.map(CKRecordID::from_payload),
            lookup_info: payload
                .lookup_info
                .map(CKUserIdentityLookupInfo::from_payload),
            name_components: payload
                .name_components
                .map(CKPersonNameComponents::from_payload),
            hasi_cloud_account: payload.hasi_cloud_account,
            contact_identifiers: payload.contact_identifiers,
        }
    }
}
