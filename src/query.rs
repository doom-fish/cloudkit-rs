use crate::private::{CKQueryPayload, SortDescriptorPayload};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SortDescriptor {
    key: String,
    ascending: bool,
}

impl SortDescriptor {
    pub fn new(key: impl Into<String>, ascending: bool) -> Self {
        Self {
            key: key.into(),
            ascending,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub const fn ascending(&self) -> bool {
        self.ascending
    }

    pub(crate) fn to_payload(&self) -> SortDescriptorPayload {
        SortDescriptorPayload {
            key: self.key.clone(),
            ascending: self.ascending,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKLocationSortDescriptor {
    key: String,
    relative_latitude: f64,
    relative_longitude: f64,
}

impl CKLocationSortDescriptor {
    pub fn new(
        key: impl Into<String>,
        relative_latitude: f64,
        relative_longitude: f64,
    ) -> Self {
        Self {
            key: key.into(),
            relative_latitude,
            relative_longitude,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub const fn relative_latitude(&self) -> f64 {
        self.relative_latitude
    }

    pub const fn relative_longitude(&self) -> f64 {
        self.relative_longitude
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKQuery {
    record_type: String,
    predicate_format: String,
    sort_descriptors: Vec<SortDescriptor>,
}

impl CKQuery {
    pub fn new(record_type: impl Into<String>, predicate_format: impl Into<String>) -> Self {
        Self {
            record_type: record_type.into(),
            predicate_format: predicate_format.into(),
            sort_descriptors: Vec::new(),
        }
    }

    pub fn match_all(record_type: impl Into<String>) -> Self {
        Self::new(record_type, "TRUEPREDICATE")
    }

    pub fn record_type(&self) -> &str {
        &self.record_type
    }

    pub fn predicate_format(&self) -> &str {
        &self.predicate_format
    }

    pub fn sort_descriptors(&self) -> &[SortDescriptor] {
        &self.sort_descriptors
    }

    pub fn with_sort_descriptor(mut self, descriptor: SortDescriptor) -> Self {
        self.sort_descriptors.push(descriptor);
        self
    }

    pub fn set_sort_descriptors(&mut self, descriptors: Vec<SortDescriptor>) {
        self.sort_descriptors = descriptors;
    }

    pub(crate) fn to_payload(&self) -> CKQueryPayload {
        CKQueryPayload {
            record_type: self.record_type.clone(),
            predicate_format: self.predicate_format.clone(),
            sort_descriptors: self
                .sort_descriptors
                .iter()
                .map(SortDescriptor::to_payload)
                .collect(),
        }
    }
}
