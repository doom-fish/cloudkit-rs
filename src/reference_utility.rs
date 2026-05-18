use crate::private::CKReferencePayload;
use crate::record::CKRecordID;

/// Mirrors `CKReferenceAction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKReferenceAction {
    /// Mirrors `CKReferenceAction.none`.
    None,
    /// Mirrors `CKReferenceAction.deleteSelf`.
    DeleteSelf,
    /// Mirrors `CKReferenceAction.unknown`.
    Unknown(u64),
}

impl CKReferenceAction {
    pub(crate) const fn from_raw(raw: u64) -> Self {
        match raw {
            0 => Self::None,
            1 => Self::DeleteSelf,
            other => Self::Unknown(other),
        }
    }

    pub(crate) const fn to_raw(self) -> u64 {
        match self {
            Self::None => 0,
            Self::DeleteSelf => 1,
            Self::Unknown(raw) => raw,
        }
    }
}

/// Wraps `CKReference`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKReference {
    record_id: CKRecordID,
    action: CKReferenceAction,
}

impl CKReference {
    /// Creates a wrapper mirroring `CKReference`.
    pub fn new(record_id: CKRecordID, action: CKReferenceAction) -> Self {
        Self { record_id, action }
    }

    /// Mirrors `CKReference.recordID`.
    pub fn record_id(&self) -> &CKRecordID {
        &self.record_id
    }

    /// Mirrors `CKReference.action`.
    pub const fn action(&self) -> CKReferenceAction {
        self.action
    }

    /// Mirrors `CKReference.parent`.
    pub fn parent(record_id: CKRecordID) -> Self {
        Self::new(record_id, CKReferenceAction::None)
    }

    /// Mirrors `CKReference.deleteSelf`.
    pub fn delete_self(record_id: CKRecordID) -> Self {
        Self::new(record_id, CKReferenceAction::DeleteSelf)
    }

    pub(crate) fn from_payload(payload: CKReferencePayload) -> Self {
        Self::new(
            CKRecordID::from_payload(payload.record_id),
            CKReferenceAction::from_raw(payload.action),
        )
    }

    pub(crate) fn to_payload(&self) -> CKReferencePayload {
        CKReferencePayload {
            record_id: self.record_id.to_payload(),
            action: self.action.to_raw(),
        }
    }
}
