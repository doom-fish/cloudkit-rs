use core::fmt;

use serde::{Deserialize, Serialize};

/// Mirrors `CKErrorDomain`.
pub const CLOUDKIT_ERROR_DOMAIN: &str = "CKErrorDomain";
/// Mirrors `CloudKitBridge`.
pub const CLOUDKIT_BRIDGE_ERROR_DOMAIN: &str = "CloudKitBridge";

/// Mirrors `CKError.Code`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CloudKitErrorCode {
    /// Mirrors `CKError.Code.bridgeInvalidArgument`.
    BridgeInvalidArgument,
    /// Mirrors `CKError.Code.bridgeFailure`.
    BridgeFailure,
    /// Mirrors `CKError.Code.bridgeTimedOut`.
    BridgeTimedOut,
    /// Mirrors `CKError.Code.bridgeDefaultContainerUnavailable`.
    BridgeDefaultContainerUnavailable,
    /// Mirrors `CKError.Code.internalError`.
    InternalError,
    /// Mirrors `CKError.Code.partialFailure`.
    PartialFailure,
    /// Mirrors `CKError.Code.networkUnavailable`.
    NetworkUnavailable,
    /// Mirrors `CKError.Code.networkFailure`.
    NetworkFailure,
    /// Mirrors `CKError.Code.badContainer`.
    BadContainer,
    /// Mirrors `CKError.Code.serviceUnavailable`.
    ServiceUnavailable,
    /// Mirrors `CKError.Code.requestRateLimited`.
    RequestRateLimited,
    /// Mirrors `CKError.Code.missingEntitlement`.
    MissingEntitlement,
    /// Mirrors `CKError.Code.notAuthenticated`.
    NotAuthenticated,
    /// Mirrors `CKError.Code.permissionFailure`.
    PermissionFailure,
    /// Mirrors `CKError.Code.unknownItem`.
    UnknownItem,
    /// Mirrors `CKError.Code.invalidArguments`.
    InvalidArguments,
    /// Mirrors `CKError.Code.serverRecordChanged`.
    ServerRecordChanged,
    /// Mirrors `CKError.Code.operationCancelled`.
    OperationCancelled,
    /// Mirrors `CKError.Code.badDatabase`.
    BadDatabase,
    /// Mirrors `CKError.Code.zoneNotFound`.
    ZoneNotFound,
    /// Mirrors `CKError.Code.limitExceeded`.
    LimitExceeded,
    /// Mirrors `CKError.Code.unknown`.
    Unknown(i64),
}

/// Wraps `CKError` details.
#[derive(Debug, Clone, PartialEq)]
pub struct CloudKitError {
    /// Mirrors `CKError.domain`.
    pub domain: String,
    /// Mirrors `CKError.code`.
    pub code: i64,
    /// Mirrors `CKError.localizedDescription`.
    pub message: String,
    /// Mirrors `CKErrorRetryAfterKey`.
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

    /// Mirrors `CKError.code`.
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

    /// Reports whether the wrapped `CKError` matches this convenience condition.
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

    /// Reports whether the wrapped `CKError` matches this convenience condition.
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

    /// Reports whether the wrapped `CKError` matches this convenience condition.
    #[must_use]
    pub fn is_missing_entitlement(&self) -> bool {
        matches!(self.kind(), CloudKitErrorCode::MissingEntitlement)
    }

    /// Reports whether the wrapped `CKError` matches this convenience condition.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_kind_maps_known_codes() {
        let cases = [
            (-1, CloudKitErrorCode::BridgeInvalidArgument),
            (-2, CloudKitErrorCode::BridgeFailure),
            (-3, CloudKitErrorCode::BridgeTimedOut),
            (-4, CloudKitErrorCode::BridgeDefaultContainerUnavailable),
        ];

        for (code, expected) in cases {
            let error = CloudKitError::bridge(code, "bridge failure");
            assert_eq!(error.kind(), expected);
        }
    }

    #[test]
    fn framework_kind_maps_known_codes() {
        let cases = [
            (1, CloudKitErrorCode::InternalError),
            (8, CloudKitErrorCode::MissingEntitlement),
            (9, CloudKitErrorCode::NotAuthenticated),
            (27, CloudKitErrorCode::LimitExceeded),
        ];

        for (code, expected) in cases {
            let error = CloudKitError {
                domain: CLOUDKIT_ERROR_DOMAIN.into(),
                code,
                message: "framework failure".into(),
                retry_after_seconds: Some(1.5),
            };
            assert_eq!(error.kind(), expected);
        }
    }

    #[test]
    fn unknown_domain_and_codes_stay_unknown() {
        let foreign_error = CloudKitError {
            domain: "OtherDomain".into(),
            code: 999,
            message: "foreign".into(),
            retry_after_seconds: None,
        };
        let bridge_error = CloudKitError::bridge(-99, "unknown bridge code");

        assert_eq!(foreign_error.kind(), CloudKitErrorCode::Unknown(999));
        assert_eq!(bridge_error.kind(), CloudKitErrorCode::Unknown(-99));
    }

    #[test]
    fn convenience_predicates_match_expected_kinds() {
        let missing_entitlement = CloudKitError {
            domain: CLOUDKIT_ERROR_DOMAIN.into(),
            code: 8,
            message: "missing entitlement".into(),
            retry_after_seconds: None,
        };
        let not_authenticated = CloudKitError {
            domain: CLOUDKIT_ERROR_DOMAIN.into(),
            code: 9,
            message: "not authenticated".into(),
            retry_after_seconds: None,
        };
        let retryable = CloudKitError {
            domain: CLOUDKIT_ERROR_DOMAIN.into(),
            code: 7,
            message: "slow down".into(),
            retry_after_seconds: Some(2.5),
        };

        assert!(missing_entitlement.is_entitlement_or_account_issue());
        assert!(missing_entitlement.is_missing_entitlement());
        assert!(!missing_entitlement.is_not_authenticated());
        assert!(not_authenticated.is_entitlement_or_account_issue());
        assert!(not_authenticated.is_not_authenticated());
        assert!(retryable.is_retryable());
    }

    #[test]
    fn from_payload_and_display_preserve_fields() {
        let error = CloudKitError::from_payload(ErrorPayload {
            domain: CLOUDKIT_ERROR_DOMAIN.into(),
            code: 7,
            message: "retry later".into(),
            retry_after_seconds: Some(2.5),
        });

        assert_eq!(error.retry_after_seconds, Some(2.5));
        assert_eq!(error.to_string(), "retry later (7) [CKErrorDomain]");
    }
}
