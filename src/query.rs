use core::ffi::c_char;
use core::ptr;

use crate::error::CloudKitError;
use crate::ffi;
use crate::private::{
    error_from_status, json_cstring, parse_json_ptr, CKQueryPayload, ComparisonOperatorPayload,
    PredicatePayload, SortDescriptorPayload,
};
use crate::record::{RecordLocation, RecordValue};

/// Wraps `CKQuery.sortDescriptors`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SortDescriptor {
    key: String,
    ascending: bool,
}

impl SortDescriptor {
    /// Creates a wrapper mirroring `CKQuery.sortDescriptors`.
    pub fn new(key: impl Into<String>, ascending: bool) -> Self {
        Self {
            key: key.into(),
            ascending,
        }
    }

    /// Mirrors `CKQuery.sortDescriptors.key`.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Mirrors `CKQuery.sortDescriptors.ascending`.
    pub const fn ascending(&self) -> bool {
        self.ascending
    }

    pub(crate) fn to_payload(&self) -> SortDescriptorPayload {
        SortDescriptorPayload {
            key: self.key.clone(),
            ascending: self.ascending,
            relative_location: None,
        }
    }
}

/// Wraps `CKLocationSortDescriptor`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKLocationSortDescriptor {
    key: String,
    relative_latitude: f64,
    relative_longitude: f64,
}

impl CKLocationSortDescriptor {
    /// Creates a wrapper mirroring `CKLocationSortDescriptor`.
    pub fn new(key: impl Into<String>, relative_latitude: f64, relative_longitude: f64) -> Self {
        Self {
            key: key.into(),
            relative_latitude,
            relative_longitude,
        }
    }

    /// Mirrors `CKLocationSortDescriptor.key`.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Mirrors `CKLocationSortDescriptor.relativeLatitude`.
    pub const fn relative_latitude(&self) -> f64 {
        self.relative_latitude
    }

    /// Mirrors `CKLocationSortDescriptor.relativeLongitude`.
    pub const fn relative_longitude(&self) -> f64 {
        self.relative_longitude
    }

    pub(crate) fn to_payload(&self) -> SortDescriptorPayload {
        SortDescriptorPayload {
            key: self.key.clone(),
            ascending: true,
            relative_location: Some(
                RecordLocation::new(self.relative_latitude, self.relative_longitude).to_payload(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum QuerySortDescriptor {
    Key(SortDescriptor),
    Location(CKLocationSortDescriptor),
}

impl QuerySortDescriptor {
    pub fn key(&self) -> &str {
        match self {
            Self::Key(descriptor) => descriptor.key(),
            Self::Location(descriptor) => descriptor.key(),
        }
    }

    pub(crate) fn to_payload(&self) -> SortDescriptorPayload {
        match self {
            Self::Key(descriptor) => descriptor.to_payload(),
            Self::Location(descriptor) => descriptor.to_payload(),
        }
    }
}

impl From<SortDescriptor> for QuerySortDescriptor {
    fn from(value: SortDescriptor) -> Self {
        Self::Key(value)
    }
}

impl From<CKLocationSortDescriptor> for QuerySortDescriptor {
    fn from(value: CKLocationSortDescriptor) -> Self {
        Self::Location(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl ComparisonOperator {
    const fn to_payload(self) -> ComparisonOperatorPayload {
        match self {
            Self::Equal => ComparisonOperatorPayload::Equal,
            Self::NotEqual => ComparisonOperatorPayload::NotEqual,
            Self::LessThan => ComparisonOperatorPayload::LessThan,
            Self::LessThanOrEqual => ComparisonOperatorPayload::LessThanOrEqual,
            Self::GreaterThan => ComparisonOperatorPayload::GreaterThan,
            Self::GreaterThanOrEqual => ComparisonOperatorPayload::GreaterThanOrEqual,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum QueryPredicate {
    True,
    False,
    Compare {
        key: String,
        operator: ComparisonOperator,
        value: RecordValue,
    },
    BeginsWith {
        key: String,
        prefix: String,
    },
    In {
        key: String,
        values: Vec<RecordValue>,
    },
    NotIn {
        key: String,
        values: Vec<RecordValue>,
    },
    ListContains {
        key: String,
        value: RecordValue,
    },
    Between {
        key: String,
        lower: RecordValue,
        upper: RecordValue,
    },
    ContainsTokens(String),
    WithinDistance {
        key: String,
        latitude: f64,
        longitude: f64,
        radius_kilometers: f64,
    },
    And(Vec<QueryPredicate>),
    Not(Box<QueryPredicate>),
    Format {
        format: String,
        arguments: Vec<RecordValue>,
    },
}

impl QueryPredicate {
    pub fn compare(
        key: impl Into<String>,
        operator: ComparisonOperator,
        value: impl Into<RecordValue>,
    ) -> Self {
        Self::Compare {
            key: key.into(),
            operator,
            value: value.into(),
        }
    }

    pub fn begins_with(key: impl Into<String>, prefix: impl Into<String>) -> Self {
        Self::BeginsWith {
            key: key.into(),
            prefix: prefix.into(),
        }
    }

    pub fn is_in<V: Into<RecordValue>>(
        key: impl Into<String>,
        values: impl IntoIterator<Item = V>,
    ) -> Self {
        Self::In {
            key: key.into(),
            values: values.into_iter().map(Into::into).collect(),
        }
    }

    pub fn not_in<V: Into<RecordValue>>(
        key: impl Into<String>,
        values: impl IntoIterator<Item = V>,
    ) -> Self {
        Self::NotIn {
            key: key.into(),
            values: values.into_iter().map(Into::into).collect(),
        }
    }

    pub fn list_contains(key: impl Into<String>, value: impl Into<RecordValue>) -> Self {
        Self::ListContains {
            key: key.into(),
            value: value.into(),
        }
    }

    pub fn between(
        key: impl Into<String>,
        lower: impl Into<RecordValue>,
        upper: impl Into<RecordValue>,
    ) -> Self {
        Self::Between {
            key: key.into(),
            lower: lower.into(),
            upper: upper.into(),
        }
    }

    pub fn contains_tokens(tokens: impl Into<String>) -> Self {
        Self::ContainsTokens(tokens.into())
    }

    pub fn within_distance(
        key: impl Into<String>,
        latitude: f64,
        longitude: f64,
        radius_kilometers: f64,
    ) -> Self {
        Self::WithinDistance {
            key: key.into(),
            latitude,
            longitude,
            radius_kilometers,
        }
    }

    pub fn and(predicates: impl IntoIterator<Item = Self>) -> Self {
        Self::And(predicates.into_iter().collect())
    }

    pub fn not(predicate: Self) -> Self {
        Self::Not(Box::new(predicate))
    }

    pub fn format(format: impl Into<String>, arguments: Vec<RecordValue>) -> Self {
        Self::Format {
            format: format.into(),
            arguments,
        }
    }

    pub(crate) fn to_payload(&self) -> PredicatePayload {
        match self {
            Self::True => PredicatePayload::TruePredicate,
            Self::False => PredicatePayload::FalsePredicate,
            Self::Compare {
                key,
                operator,
                value,
            } => PredicatePayload::Comparison {
                key: key.clone(),
                comparison: operator.to_payload(),
                value: value.to_payload(),
            },
            Self::BeginsWith { key, prefix } => PredicatePayload::BeginsWith {
                key: key.clone(),
                text: prefix.clone(),
            },
            Self::In { key, values } => PredicatePayload::IsIn {
                key: key.clone(),
                values: values.iter().map(RecordValue::to_payload).collect(),
            },
            Self::NotIn { key, values } => PredicatePayload::NotIn {
                key: key.clone(),
                values: values.iter().map(RecordValue::to_payload).collect(),
            },
            Self::ListContains { key, value } => PredicatePayload::ListContains {
                key: key.clone(),
                value: value.to_payload(),
            },
            Self::Between { key, lower, upper } => PredicatePayload::Between {
                key: key.clone(),
                values: vec![lower.to_payload(), upper.to_payload()],
            },
            Self::ContainsTokens(tokens) => PredicatePayload::ContainsTokens {
                text: tokens.clone(),
            },
            Self::WithinDistance {
                key,
                latitude,
                longitude,
                radius_kilometers,
            } => PredicatePayload::WithinDistance {
                key: key.clone(),
                location: RecordLocation::new(*latitude, *longitude).to_payload(),
                radius_kilometers: *radius_kilometers,
            },
            Self::And(predicates) if predicates.is_empty() => PredicatePayload::TruePredicate,
            Self::And(predicates) => PredicatePayload::And {
                subpredicates: predicates.iter().map(Self::to_payload).collect(),
            },
            Self::Not(predicate) => PredicatePayload::Not {
                subpredicates: vec![predicate.to_payload()],
            },
            Self::Format { format, arguments } => PredicatePayload::Format {
                format: format.clone(),
                arguments: arguments.iter().map(RecordValue::to_payload).collect(),
            },
        }
    }
}

/// Wraps `CKQuery`.
#[derive(Debug, Clone, PartialEq)]
pub struct CKQuery {
    record_type: String,
    predicate: QueryPredicate,
    sort_descriptors: Vec<QuerySortDescriptor>,
}

impl CKQuery {
    /// Creates a wrapper mirroring `CKQuery`.
    pub fn new(record_type: impl Into<String>, predicate: QueryPredicate) -> Self {
        Self {
            record_type: record_type.into(),
            predicate,
            sort_descriptors: Vec::new(),
        }
    }

    /// Mirrors `CKQuery.matchAll`.
    pub fn match_all(record_type: impl Into<String>) -> Self {
        Self::new(record_type, QueryPredicate::True)
    }

    /// Mirrors `CKQuery.recordType`.
    pub fn record_type(&self) -> &str {
        &self.record_type
    }

    pub const fn predicate(&self) -> &QueryPredicate {
        &self.predicate
    }

    /// Mirrors `CKQuery.predicateFormat`.
    pub fn predicate_format(&self) -> Result<String, CloudKitError> {
        let query_json = json_cstring(&self.to_payload(), "query")?;
        let mut out_json: *mut c_char = ptr::null_mut();
        let mut out_error: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::ck_query_predicate_format(
                query_json.as_ptr(),
                &raw mut out_json,
                &raw mut out_error,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { error_from_status(status, out_error) });
        }
        unsafe { parse_json_ptr::<String>(out_json, "predicate format") }
    }

    /// Mirrors `CKQuery.sortDescriptors`.
    pub fn sort_descriptors(&self) -> &[QuerySortDescriptor] {
        &self.sort_descriptors
    }

    /// Sets the value mirroring `CKQuery.sortDescriptor`.
    pub fn with_sort_descriptor(mut self, descriptor: impl Into<QuerySortDescriptor>) -> Self {
        self.sort_descriptors.push(descriptor.into());
        self
    }

    /// Mirrors `CKQuery.setSortDescriptors`.
    pub fn set_sort_descriptors<D: Into<QuerySortDescriptor>>(
        &mut self,
        descriptors: impl IntoIterator<Item = D>,
    ) {
        self.sort_descriptors = descriptors.into_iter().map(Into::into).collect();
    }

    pub(crate) fn to_payload(&self) -> CKQueryPayload {
        CKQueryPayload {
            record_type: self.record_type.clone(),
            predicate: self.predicate.to_payload(),
            sort_descriptors: self
                .sort_descriptors
                .iter()
                .map(QuerySortDescriptor::to_payload)
                .collect(),
        }
    }
}
