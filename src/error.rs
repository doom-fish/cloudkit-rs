use core::fmt;

use serde::{Deserialize, Serialize};

pub const CLOUDKIT_ERROR_DOMAIN: &str = "CKErrorDomain";
pub const CLOUDKIT_BRIDGE_ERROR_DOMAIN: &str = "CloudKitBridge";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CloudKitErrorCode {
    BridgeInvalidArgument,
    BridgeFailure,
    BridgeTimedOut,
    BridgeDefaultContainerUnavailable,
    InternalError,
    PartialFailure,
    NetworkUnavailable,
    NetworkFailure,
    BadContainer,
    ServiceUnavailable,
    RequestRateLimited,
    MissingEntitlement,
    NotAuthenticated,
    PermissionFailure,
    UnknownItem,
    InvalidArguments,
    ServerRecordChanged,
    OperationCancelled,
    BadDatabase,
    ZoneNotFound,
    LimitExceeded,
    Unknown(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CloudKitError {
    pub domain: String,
    pub code: i64,
    pub message: String,
    pub retry_after_seconds: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ErrorPayload {
    pub domain: String,
    pub code: i64,
    pub message: String,
    pub retry_after_seconds: Option<f64>,
}

impl CloudKitError {
    pub(crate) fn from_payload(payload: ErrorPayload) -> Self {
        Self {
            domain: payload.domain,
            code: payload.code,
            message: payload.message,
            retry_after_seconds: payload.retry_after_seconds,
        }
    }

    pub(crate) fn bridge(code: i64, message: impl Into<String>) -> Self {
        Self {
            domain: CLOUDKIT_BRIDGE_ERROR_DOMAIN.into(),
            code,
            message: message.into(),
            retry_after_seconds: None,
        }
    }

    #[must_use]
    pub fn kind(&self) -> CloudKitErrorCode {
        if self.domain == CLOUDKIT_BRIDGE_ERROR_DOMAIN {
            return match self.code {
                -1 => CloudKitErrorCode::BridgeInvalidArgument,
                -2 => CloudKitErrorCode::BridgeFailure,
                -3 => CloudKitErrorCode::BridgeTimedOut,
                -4 => CloudKitErrorCode::BridgeDefaultContainerUnavailable,
                other => CloudKitErrorCode::Unknown(other),
            };
        }

        if self.domain != CLOUDKIT_ERROR_DOMAIN {
            return CloudKitErrorCode::Unknown(self.code);
        }

        match self.code {
            1 => CloudKitErrorCode::InternalError,
            2 => CloudKitErrorCode::PartialFailure,
            3 => CloudKitErrorCode::NetworkUnavailable,
            4 => CloudKitErrorCode::NetworkFailure,
            5 => CloudKitErrorCode::BadContainer,
            6 => CloudKitErrorCode::ServiceUnavailable,
            7 => CloudKitErrorCode::RequestRateLimited,
            8 => CloudKitErrorCode::MissingEntitlement,
            9 => CloudKitErrorCode::NotAuthenticated,
            10 => CloudKitErrorCode::PermissionFailure,
            11 => CloudKitErrorCode::UnknownItem,
            12 => CloudKitErrorCode::InvalidArguments,
            14 => CloudKitErrorCode::ServerRecordChanged,
            20 => CloudKitErrorCode::OperationCancelled,
            24 => CloudKitErrorCode::BadDatabase,
            26 => CloudKitErrorCode::ZoneNotFound,
            27 => CloudKitErrorCode::LimitExceeded,
            other => CloudKitErrorCode::Unknown(other),
        }
    }

    #[must_use]
    pub fn is_entitlement_or_account_issue(&self) -> bool {
        matches!(
            self.kind(),
            CloudKitErrorCode::BridgeDefaultContainerUnavailable
                | CloudKitErrorCode::BadContainer
                | CloudKitErrorCode::MissingEntitlement
                | CloudKitErrorCode::NotAuthenticated
                | CloudKitErrorCode::PermissionFailure
        )
    }

    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind(),
            CloudKitErrorCode::NetworkUnavailable
                | CloudKitErrorCode::NetworkFailure
                | CloudKitErrorCode::ServiceUnavailable
                | CloudKitErrorCode::RequestRateLimited
        )
    }

    #[must_use]
    pub fn is_missing_entitlement(&self) -> bool {
        matches!(self.kind(), CloudKitErrorCode::MissingEntitlement)
    }

    #[must_use]
    pub fn is_not_authenticated(&self) -> bool {
        matches!(self.kind(), CloudKitErrorCode::NotAuthenticated)
    }
}

impl fmt::Display for CloudKitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}) [{}]", self.message, self.code, self.domain)
    }
}

impl std::error::Error for CloudKitError {}
