use crate::private::CKServerChangeTokenPayload;

/// Wraps `CKServerChangeToken`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKServerChangeToken {
    archived_data: Vec<u8>,
}

impl CKServerChangeToken {
    /// Creates this wrapper from archived `CKServerChangeToken` data.
    pub fn from_archived_data(archived_data: Vec<u8>) -> Self {
        Self { archived_data }
    }

    /// Mirrors archived data stored by `CKServerChangeToken`.
    pub fn archived_data(&self) -> &[u8] {
        &self.archived_data
    }

    pub(crate) fn from_payload(payload: CKServerChangeTokenPayload) -> Self {
        Self::from_archived_data(payload.archived_data)
    }

    pub(crate) fn to_payload(&self) -> CKServerChangeTokenPayload {
        CKServerChangeTokenPayload {
            archived_data: self.archived_data.clone(),
        }
    }
}
