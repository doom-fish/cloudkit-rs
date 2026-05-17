use std::fmt;
use std::sync::Arc;

use crate::database::CKDatabase;
use crate::error::CloudKitError;
use crate::operation::CKOperationGroup;
use crate::record::{CKRecord, CKRecordID, CKRecordZone, CKRecordZoneID};
use crate::user_identity::CKUserIdentity;

fn push_unique<T>(values: &mut Vec<T>, value: T)
where
    T: PartialEq,
{
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKSyncEngineSyncReason {
    Scheduled,
    Manual,
    Unknown(i32),
}

impl CKSyncEngineSyncReason {
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Scheduled,
            1 => Self::Manual,
            other => Self::Unknown(other),
        }
    }

    pub const fn to_raw(self) -> i32 {
        match self {
            Self::Scheduled => 0,
            Self::Manual => 1,
            Self::Unknown(raw) => raw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct CKSyncEngineStateSerialization {
    archived_data: Vec<u8>,
}

impl CKSyncEngineStateSerialization {
    pub fn new(archived_data: Vec<u8>) -> Self {
        Self { archived_data }
    }

    pub fn archived_data(&self) -> &[u8] {
        &self.archived_data
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKSyncEngineFetchChangesScope {
    zone_ids: Vec<CKRecordZoneID>,
    excluded_zone_ids: Vec<CKRecordZoneID>,
}

impl CKSyncEngineFetchChangesScope {
    pub fn new() -> Self {
        Self {
            zone_ids: Vec::new(),
            excluded_zone_ids: Vec::new(),
        }
    }

    pub fn with_zone_ids(zone_ids: Vec<CKRecordZoneID>) -> Self {
        Self {
            zone_ids,
            excluded_zone_ids: Vec::new(),
        }
    }

    pub fn with_excluded_zone_ids(excluded_zone_ids: Vec<CKRecordZoneID>) -> Self {
        Self {
            zone_ids: Vec::new(),
            excluded_zone_ids,
        }
    }

    pub fn zone_ids(&self) -> &[CKRecordZoneID] {
        &self.zone_ids
    }

    pub fn excluded_zone_ids(&self) -> &[CKRecordZoneID] {
        &self.excluded_zone_ids
    }

    pub fn add_zone_id(mut self, zone_id: CKRecordZoneID) -> Self {
        push_unique(&mut self.zone_ids, zone_id);
        self
    }

    pub fn add_excluded_zone_id(mut self, zone_id: CKRecordZoneID) -> Self {
        push_unique(&mut self.excluded_zone_ids, zone_id);
        self
    }

    pub fn contains_zone_id(&self, zone_id: &CKRecordZoneID) -> bool {
        let included =
            self.zone_ids.is_empty() || self.zone_ids.iter().any(|candidate| candidate == zone_id);
        included
            && !self
                .excluded_zone_ids
                .iter()
                .any(|candidate| candidate == zone_id)
    }
}

impl Default for CKSyncEngineFetchChangesScope {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineFetchChangesOptions {
    scope: CKSyncEngineFetchChangesScope,
    operation_group: Option<CKOperationGroup>,
    prioritized_zone_ids: Vec<CKRecordZoneID>,
}

impl CKSyncEngineFetchChangesOptions {
    pub fn new(scope: CKSyncEngineFetchChangesScope) -> Self {
        Self {
            scope,
            operation_group: None,
            prioritized_zone_ids: Vec::new(),
        }
    }

    pub const fn scope(&self) -> &CKSyncEngineFetchChangesScope {
        &self.scope
    }

    pub const fn operation_group(&self) -> Option<&CKOperationGroup> {
        self.operation_group.as_ref()
    }

    pub fn prioritized_zone_ids(&self) -> &[CKRecordZoneID] {
        &self.prioritized_zone_ids
    }

    pub fn with_operation_group(mut self, operation_group: CKOperationGroup) -> Self {
        self.operation_group = Some(operation_group);
        self
    }

    pub fn with_prioritized_zone_id(mut self, zone_id: CKRecordZoneID) -> Self {
        push_unique(&mut self.prioritized_zone_ids, zone_id);
        self
    }
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKSyncEngineSendChangesScope {
    zone_ids: Vec<CKRecordZoneID>,
    excluded_zone_ids: Vec<CKRecordZoneID>,
    record_ids: Vec<CKRecordID>,
}

impl CKSyncEngineSendChangesScope {
    pub fn new() -> Self {
        Self {
            zone_ids: Vec::new(),
            excluded_zone_ids: Vec::new(),
            record_ids: Vec::new(),
        }
    }

    pub fn with_zone_ids(zone_ids: Vec<CKRecordZoneID>) -> Self {
        Self {
            zone_ids,
            excluded_zone_ids: Vec::new(),
            record_ids: Vec::new(),
        }
    }

    pub fn with_excluded_zone_ids(excluded_zone_ids: Vec<CKRecordZoneID>) -> Self {
        Self {
            zone_ids: Vec::new(),
            excluded_zone_ids,
            record_ids: Vec::new(),
        }
    }

    pub fn with_record_ids(record_ids: Vec<CKRecordID>) -> Self {
        Self {
            zone_ids: Vec::new(),
            excluded_zone_ids: Vec::new(),
            record_ids,
        }
    }

    pub fn zone_ids(&self) -> &[CKRecordZoneID] {
        &self.zone_ids
    }

    pub fn excluded_zone_ids(&self) -> &[CKRecordZoneID] {
        &self.excluded_zone_ids
    }

    pub fn record_ids(&self) -> &[CKRecordID] {
        &self.record_ids
    }

    pub fn add_zone_id(mut self, zone_id: CKRecordZoneID) -> Self {
        push_unique(&mut self.zone_ids, zone_id);
        self
    }

    pub fn add_excluded_zone_id(mut self, zone_id: CKRecordZoneID) -> Self {
        push_unique(&mut self.excluded_zone_ids, zone_id);
        self
    }

    pub fn add_record_id(mut self, record_id: CKRecordID) -> Self {
        push_unique(&mut self.record_ids, record_id);
        self
    }

    pub fn contains_zone_id(&self, zone_id: &CKRecordZoneID) -> bool {
        let included =
            self.zone_ids.is_empty() || self.zone_ids.iter().any(|candidate| candidate == zone_id);
        included
            && !self
                .excluded_zone_ids
                .iter()
                .any(|candidate| candidate == zone_id)
    }

    pub fn contains_record_id(&self, record_id: &CKRecordID) -> bool {
        let zone_match = self.contains_zone_id(record_id.zone_id());
        let record_match = self.record_ids.is_empty()
            || self
                .record_ids
                .iter()
                .any(|candidate| candidate == record_id);
        zone_match && record_match
    }

    pub fn contains_pending_record_zone_change(
        &self,
        pending_change: &CKSyncEnginePendingRecordZoneChange,
    ) -> bool {
        self.contains_record_id(pending_change.record_id())
    }
}

impl Default for CKSyncEngineSendChangesScope {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineSendChangesOptions {
    scope: CKSyncEngineSendChangesScope,
    operation_group: Option<CKOperationGroup>,
}

impl CKSyncEngineSendChangesOptions {
    pub fn new(scope: CKSyncEngineSendChangesScope) -> Self {
        Self {
            scope,
            operation_group: None,
        }
    }

    pub const fn scope(&self) -> &CKSyncEngineSendChangesScope {
        &self.scope
    }

    pub const fn operation_group(&self) -> Option<&CKOperationGroup> {
        self.operation_group.as_ref()
    }

    pub fn with_operation_group(mut self, operation_group: CKOperationGroup) -> Self {
        self.operation_group = Some(operation_group);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineFetchChangesContext {
    reason: CKSyncEngineSyncReason,
    options: CKSyncEngineFetchChangesOptions,
}

impl CKSyncEngineFetchChangesContext {
    pub fn new(reason: CKSyncEngineSyncReason, options: CKSyncEngineFetchChangesOptions) -> Self {
        Self { reason, options }
    }

    pub const fn reason(&self) -> CKSyncEngineSyncReason {
        self.reason
    }

    pub const fn options(&self) -> &CKSyncEngineFetchChangesOptions {
        &self.options
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineSendChangesContext {
    reason: CKSyncEngineSyncReason,
    options: CKSyncEngineSendChangesOptions,
}

impl CKSyncEngineSendChangesContext {
    pub fn new(reason: CKSyncEngineSyncReason, options: CKSyncEngineSendChangesOptions) -> Self {
        Self { reason, options }
    }

    pub const fn reason(&self) -> CKSyncEngineSyncReason {
        self.reason
    }

    pub const fn options(&self) -> &CKSyncEngineSendChangesOptions {
        &self.options
    }
}

pub trait CKSyncEngineDelegate: Send + Sync {
    fn handle_event(&self, engine: &CKSyncEngine, event: &CKSyncEngineEvent);

    fn next_record_zone_change_batch_for_context(
        &self,
        engine: &CKSyncEngine,
        context: &CKSyncEngineSendChangesContext,
    ) -> Option<CKSyncEngineRecordZoneChangeBatch>;

    fn next_fetch_changes_options_for_context(
        &self,
        _engine: &CKSyncEngine,
        _context: &CKSyncEngineFetchChangesContext,
    ) -> Option<CKSyncEngineFetchChangesOptions> {
        None
    }
}

type CKSyncEngineDelegateRef = Arc<dyn CKSyncEngineDelegate>;

#[derive(Clone)]
pub struct CKSyncEngineConfiguration {
    database: CKDatabase,
    state_serialization: Option<CKSyncEngineStateSerialization>,
    delegate: CKSyncEngineDelegateRef,
    automatically_sync: bool,
    subscription_id: Option<String>,
}

impl fmt::Debug for CKSyncEngineConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CKSyncEngineConfiguration")
            .field("database", &self.database)
            .field("state_serialization", &self.state_serialization)
            .field("automatically_sync", &self.automatically_sync)
            .field("subscription_id", &self.subscription_id)
            .finish_non_exhaustive()
    }
}

impl CKSyncEngineConfiguration {
    pub fn new(
        database: CKDatabase,
        state_serialization: Option<CKSyncEngineStateSerialization>,
        delegate: CKSyncEngineDelegateRef,
    ) -> Self {
        Self {
            database,
            state_serialization,
            delegate,
            automatically_sync: true,
            subscription_id: None,
        }
    }

    pub const fn database(&self) -> &CKDatabase {
        &self.database
    }

    pub const fn state_serialization(&self) -> Option<&CKSyncEngineStateSerialization> {
        self.state_serialization.as_ref()
    }

    pub fn delegate(&self) -> CKSyncEngineDelegateRef {
        Arc::clone(&self.delegate)
    }

    pub const fn automatically_sync(&self) -> bool {
        self.automatically_sync
    }

    pub fn subscription_id(&self) -> Option<&str> {
        self.subscription_id.as_deref()
    }

    pub fn with_state_serialization(
        mut self,
        state_serialization: CKSyncEngineStateSerialization,
    ) -> Self {
        self.state_serialization = Some(state_serialization);
        self
    }

    pub fn with_automatically_sync(mut self, automatically_sync: bool) -> Self {
        self.automatically_sync = automatically_sync;
        self
    }

    pub fn with_subscription_id(mut self, subscription_id: impl Into<String>) -> Self {
        self.subscription_id = Some(subscription_id.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKSyncEnginePendingRecordZoneChangeType {
    SaveRecord,
    DeleteRecord,
    Unknown(i32),
}

impl CKSyncEnginePendingRecordZoneChangeType {
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::SaveRecord,
            1 => Self::DeleteRecord,
            other => Self::Unknown(other),
        }
    }

    pub const fn to_raw(self) -> i32 {
        match self {
            Self::SaveRecord => 0,
            Self::DeleteRecord => 1,
            Self::Unknown(raw) => raw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKSyncEnginePendingRecordZoneChange {
    record_id: CKRecordID,
    change_type: CKSyncEnginePendingRecordZoneChangeType,
}

impl CKSyncEnginePendingRecordZoneChange {
    pub fn new(
        record_id: CKRecordID,
        change_type: CKSyncEnginePendingRecordZoneChangeType,
    ) -> Self {
        Self {
            record_id,
            change_type,
        }
    }

    pub const fn record_id(&self) -> &CKRecordID {
        &self.record_id
    }

    pub const fn change_type(&self) -> CKSyncEnginePendingRecordZoneChangeType {
        self.change_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKSyncEnginePendingDatabaseChangeType {
    SaveZone,
    DeleteZone,
    Unknown(i32),
}

impl CKSyncEnginePendingDatabaseChangeType {
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::SaveZone,
            1 => Self::DeleteZone,
            other => Self::Unknown(other),
        }
    }

    pub const fn to_raw(self) -> i32 {
        match self {
            Self::SaveZone => 0,
            Self::DeleteZone => 1,
            Self::Unknown(raw) => raw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKSyncEnginePendingDatabaseChange {
    zone_id: CKRecordZoneID,
    change_type: CKSyncEnginePendingDatabaseChangeType,
}

impl CKSyncEnginePendingDatabaseChange {
    pub fn new(
        zone_id: CKRecordZoneID,
        change_type: CKSyncEnginePendingDatabaseChangeType,
    ) -> Self {
        Self {
            zone_id,
            change_type,
        }
    }

    pub const fn zone_id(&self) -> &CKRecordZoneID {
        &self.zone_id
    }

    pub const fn change_type(&self) -> CKSyncEnginePendingDatabaseChangeType {
        self.change_type
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKSyncEnginePendingZoneSave {
    pending_change: CKSyncEnginePendingDatabaseChange,
    zone: CKRecordZone,
}

impl CKSyncEnginePendingZoneSave {
    pub fn new(zone: CKRecordZone) -> Self {
        Self {
            pending_change: CKSyncEnginePendingDatabaseChange::new(
                zone.zone_id().clone(),
                CKSyncEnginePendingDatabaseChangeType::SaveZone,
            ),
            zone,
        }
    }

    pub const fn pending_change(&self) -> &CKSyncEnginePendingDatabaseChange {
        &self.pending_change
    }

    pub const fn zone(&self) -> &CKRecordZone {
        &self.zone
    }

    pub fn into_pending_change(self) -> CKSyncEnginePendingDatabaseChange {
        self.pending_change
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKSyncEnginePendingZoneDelete {
    pending_change: CKSyncEnginePendingDatabaseChange,
}

impl CKSyncEnginePendingZoneDelete {
    pub fn new(zone_id: CKRecordZoneID) -> Self {
        Self {
            pending_change: CKSyncEnginePendingDatabaseChange::new(
                zone_id,
                CKSyncEnginePendingDatabaseChangeType::DeleteZone,
            ),
        }
    }

    pub const fn pending_change(&self) -> &CKSyncEnginePendingDatabaseChange {
        &self.pending_change
    }

    pub const fn zone_id(&self) -> &CKRecordZoneID {
        self.pending_change.zone_id()
    }

    pub fn into_pending_change(self) -> CKSyncEnginePendingDatabaseChange {
        self.pending_change
    }
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CKSyncEngineState {
    pending_record_zone_changes: Vec<CKSyncEnginePendingRecordZoneChange>,
    pending_database_changes: Vec<CKSyncEnginePendingDatabaseChange>,
    has_pending_untracked_changes: bool,
    zone_ids_with_unfetched_server_changes: Vec<CKRecordZoneID>,
}

impl CKSyncEngineState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pending_record_zone_changes(&self) -> &[CKSyncEnginePendingRecordZoneChange] {
        &self.pending_record_zone_changes
    }

    pub fn pending_database_changes(&self) -> &[CKSyncEnginePendingDatabaseChange] {
        &self.pending_database_changes
    }

    pub const fn has_pending_untracked_changes(&self) -> bool {
        self.has_pending_untracked_changes
    }

    pub fn zone_ids_with_unfetched_server_changes(&self) -> &[CKRecordZoneID] {
        &self.zone_ids_with_unfetched_server_changes
    }

    pub fn add_pending_record_zone_change(
        &mut self,
        pending_change: CKSyncEnginePendingRecordZoneChange,
    ) {
        push_unique(&mut self.pending_record_zone_changes, pending_change);
    }

    pub fn remove_pending_record_zone_change(
        &mut self,
        record_id: &CKRecordID,
        change_type: CKSyncEnginePendingRecordZoneChangeType,
    ) {
        self.pending_record_zone_changes.retain(|pending_change| {
            pending_change.record_id() != record_id || pending_change.change_type() != change_type
        });
    }

    pub fn add_pending_database_change(
        &mut self,
        pending_change: CKSyncEnginePendingDatabaseChange,
    ) {
        push_unique(&mut self.pending_database_changes, pending_change);
    }

    pub fn remove_pending_database_change(
        &mut self,
        zone_id: &CKRecordZoneID,
        change_type: CKSyncEnginePendingDatabaseChangeType,
    ) {
        self.pending_database_changes.retain(|pending_change| {
            pending_change.zone_id() != zone_id || pending_change.change_type() != change_type
        });
    }

    pub fn set_has_pending_untracked_changes(&mut self, has_pending_untracked_changes: bool) {
        self.has_pending_untracked_changes = has_pending_untracked_changes;
    }

    pub fn add_zone_id_with_unfetched_server_changes(&mut self, zone_id: CKRecordZoneID) {
        push_unique(&mut self.zone_ids_with_unfetched_server_changes, zone_id);
    }

    pub fn remove_zone_id_with_unfetched_server_changes(&mut self, zone_id: &CKRecordZoneID) {
        self.zone_ids_with_unfetched_server_changes
            .retain(|candidate| candidate != zone_id);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineRecordZoneChangeBatch {
    pending_changes: Vec<CKSyncEnginePendingRecordZoneChange>,
    records_to_save: Vec<CKRecord>,
    record_ids_to_delete: Vec<CKRecordID>,
    atomic_by_zone: bool,
}

impl CKSyncEngineRecordZoneChangeBatch {
    pub fn new(
        records_to_save: Vec<CKRecord>,
        record_ids_to_delete: Vec<CKRecordID>,
        atomic_by_zone: bool,
    ) -> Self {
        Self {
            pending_changes: Vec::new(),
            records_to_save,
            record_ids_to_delete,
            atomic_by_zone,
        }
    }

    pub fn from_pending_changes(
        pending_changes: Vec<CKSyncEnginePendingRecordZoneChange>,
        records_to_save: Vec<CKRecord>,
        record_ids_to_delete: Vec<CKRecordID>,
    ) -> Self {
        Self {
            pending_changes,
            records_to_save,
            record_ids_to_delete,
            atomic_by_zone: true,
        }
    }

    pub fn pending_changes(&self) -> &[CKSyncEnginePendingRecordZoneChange] {
        &self.pending_changes
    }

    pub fn records_to_save(&self) -> &[CKRecord] {
        &self.records_to_save
    }

    pub fn record_ids_to_delete(&self) -> &[CKRecordID] {
        &self.record_ids_to_delete
    }

    pub const fn atomic_by_zone(&self) -> bool {
        self.atomic_by_zone
    }

    pub fn with_atomic_by_zone(mut self, atomic_by_zone: bool) -> Self {
        self.atomic_by_zone = atomic_by_zone;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKSyncEngineEventType {
    StateUpdate,
    AccountChange,
    FetchedDatabaseChanges,
    FetchedRecordZoneChanges,
    SentDatabaseChanges,
    SentRecordZoneChanges,
    WillFetchChanges,
    WillFetchRecordZoneChanges,
    DidFetchRecordZoneChanges,
    DidFetchChanges,
    WillSendChanges,
    DidSendChanges,
    Unknown(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKSyncEngineAccountChangeType {
    SignIn,
    SignOut,
    SwitchAccounts,
    Unknown(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CKSyncEngineZoneDeletionReason {
    Deleted,
    Purged,
    EncryptedDataReset,
    Unknown(i32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKSyncEngineFetchedRecordDeletion {
    record_id: CKRecordID,
    record_type: String,
}

impl CKSyncEngineFetchedRecordDeletion {
    pub fn new(record_id: CKRecordID, record_type: impl Into<String>) -> Self {
        Self {
            record_id,
            record_type: record_type.into(),
        }
    }

    pub const fn record_id(&self) -> &CKRecordID {
        &self.record_id
    }

    pub fn record_type(&self) -> &str {
        &self.record_type
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CKSyncEngineFetchedZoneDeletion {
    zone_id: CKRecordZoneID,
    reason: CKSyncEngineZoneDeletionReason,
}

impl CKSyncEngineFetchedZoneDeletion {
    pub fn new(zone_id: CKRecordZoneID, reason: CKSyncEngineZoneDeletionReason) -> Self {
        Self { zone_id, reason }
    }

    pub const fn zone_id(&self) -> &CKRecordZoneID {
        &self.zone_id
    }

    pub const fn reason(&self) -> CKSyncEngineZoneDeletionReason {
        self.reason
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineFailedRecordSave {
    record: CKRecord,
    error: CloudKitError,
}

impl CKSyncEngineFailedRecordSave {
    pub fn new(record: CKRecord, error: CloudKitError) -> Self {
        Self { record, error }
    }

    pub const fn record(&self) -> &CKRecord {
        &self.record
    }

    pub const fn error(&self) -> &CloudKitError {
        &self.error
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineFailedZoneSave {
    record_zone: CKRecordZone,
    error: CloudKitError,
}

impl CKSyncEngineFailedZoneSave {
    pub fn new(record_zone: CKRecordZone, error: CloudKitError) -> Self {
        Self { record_zone, error }
    }

    pub const fn record_zone(&self) -> &CKRecordZone {
        &self.record_zone
    }

    pub const fn error(&self) -> &CloudKitError {
        &self.error
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineFailedRecordDelete {
    record_id: CKRecordID,
    error: CloudKitError,
}

impl CKSyncEngineFailedRecordDelete {
    pub fn new(record_id: CKRecordID, error: CloudKitError) -> Self {
        Self { record_id, error }
    }

    pub const fn record_id(&self) -> &CKRecordID {
        &self.record_id
    }

    pub const fn error(&self) -> &CloudKitError {
        &self.error
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineFailedZoneDelete {
    zone_id: CKRecordZoneID,
    error: CloudKitError,
}

impl CKSyncEngineFailedZoneDelete {
    pub fn new(zone_id: CKRecordZoneID, error: CloudKitError) -> Self {
        Self { zone_id, error }
    }

    pub const fn zone_id(&self) -> &CKRecordZoneID {
        &self.zone_id
    }

    pub const fn error(&self) -> &CloudKitError {
        &self.error
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKSyncEngineStateUpdateEvent {
    state_serialization: CKSyncEngineStateSerialization,
}

impl CKSyncEngineStateUpdateEvent {
    pub fn new(state_serialization: CKSyncEngineStateSerialization) -> Self {
        Self {
            state_serialization,
        }
    }

    pub const fn state_serialization(&self) -> &CKSyncEngineStateSerialization {
        &self.state_serialization
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKSyncEngineAccountChangeEvent {
    change_type: CKSyncEngineAccountChangeType,
    previous_user: Option<CKUserIdentity>,
    current_user: Option<CKUserIdentity>,
}

impl CKSyncEngineAccountChangeEvent {
    pub fn new(change_type: CKSyncEngineAccountChangeType) -> Self {
        Self {
            change_type,
            previous_user: None,
            current_user: None,
        }
    }

    pub const fn change_type(&self) -> CKSyncEngineAccountChangeType {
        self.change_type
    }

    pub const fn previous_user(&self) -> Option<&CKUserIdentity> {
        self.previous_user.as_ref()
    }

    pub const fn current_user(&self) -> Option<&CKUserIdentity> {
        self.current_user.as_ref()
    }

    pub fn with_previous_user(mut self, previous_user: CKUserIdentity) -> Self {
        self.previous_user = Some(previous_user);
        self
    }

    pub fn with_current_user(mut self, current_user: CKUserIdentity) -> Self {
        self.current_user = Some(current_user);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKSyncEngineFetchedDatabaseChangesEvent {
    modifications: Vec<CKRecordZone>,
    deletions: Vec<CKSyncEngineFetchedZoneDeletion>,
}

impl CKSyncEngineFetchedDatabaseChangesEvent {
    pub fn new(
        modifications: Vec<CKRecordZone>,
        deletions: Vec<CKSyncEngineFetchedZoneDeletion>,
    ) -> Self {
        Self {
            modifications,
            deletions,
        }
    }

    pub fn modifications(&self) -> &[CKRecordZone] {
        &self.modifications
    }

    pub fn deletions(&self) -> &[CKSyncEngineFetchedZoneDeletion] {
        &self.deletions
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineFetchedRecordZoneChangesEvent {
    modifications: Vec<CKRecord>,
    deletions: Vec<CKSyncEngineFetchedRecordDeletion>,
}

impl CKSyncEngineFetchedRecordZoneChangesEvent {
    pub fn new(
        modifications: Vec<CKRecord>,
        deletions: Vec<CKSyncEngineFetchedRecordDeletion>,
    ) -> Self {
        Self {
            modifications,
            deletions,
        }
    }

    pub fn modifications(&self) -> &[CKRecord] {
        &self.modifications
    }

    pub fn deletions(&self) -> &[CKSyncEngineFetchedRecordDeletion] {
        &self.deletions
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineSentDatabaseChangesEvent {
    saved_zones: Vec<CKRecordZone>,
    failed_zone_saves: Vec<CKSyncEngineFailedZoneSave>,
    deleted_zone_ids: Vec<CKRecordZoneID>,
    failed_zone_deletes: Vec<CKSyncEngineFailedZoneDelete>,
}

impl CKSyncEngineSentDatabaseChangesEvent {
    pub fn new(
        saved_zones: Vec<CKRecordZone>,
        failed_zone_saves: Vec<CKSyncEngineFailedZoneSave>,
        deleted_zone_ids: Vec<CKRecordZoneID>,
        failed_zone_deletes: Vec<CKSyncEngineFailedZoneDelete>,
    ) -> Self {
        Self {
            saved_zones,
            failed_zone_saves,
            deleted_zone_ids,
            failed_zone_deletes,
        }
    }

    pub fn saved_zones(&self) -> &[CKRecordZone] {
        &self.saved_zones
    }

    pub fn failed_zone_saves(&self) -> &[CKSyncEngineFailedZoneSave] {
        &self.failed_zone_saves
    }

    pub fn deleted_zone_ids(&self) -> &[CKRecordZoneID] {
        &self.deleted_zone_ids
    }

    pub fn failed_zone_deletes(&self) -> &[CKSyncEngineFailedZoneDelete] {
        &self.failed_zone_deletes
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineSentRecordZoneChangesEvent {
    saved_records: Vec<CKRecord>,
    failed_record_saves: Vec<CKSyncEngineFailedRecordSave>,
    deleted_record_ids: Vec<CKRecordID>,
    failed_record_deletes: Vec<CKSyncEngineFailedRecordDelete>,
}

impl CKSyncEngineSentRecordZoneChangesEvent {
    pub fn new(
        saved_records: Vec<CKRecord>,
        failed_record_saves: Vec<CKSyncEngineFailedRecordSave>,
        deleted_record_ids: Vec<CKRecordID>,
        failed_record_deletes: Vec<CKSyncEngineFailedRecordDelete>,
    ) -> Self {
        Self {
            saved_records,
            failed_record_saves,
            deleted_record_ids,
            failed_record_deletes,
        }
    }

    pub fn saved_records(&self) -> &[CKRecord] {
        &self.saved_records
    }

    pub fn failed_record_saves(&self) -> &[CKSyncEngineFailedRecordSave] {
        &self.failed_record_saves
    }

    pub fn deleted_record_ids(&self) -> &[CKRecordID] {
        &self.deleted_record_ids
    }

    pub fn failed_record_deletes(&self) -> &[CKSyncEngineFailedRecordDelete] {
        &self.failed_record_deletes
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineWillFetchChangesEvent {
    context: CKSyncEngineFetchChangesContext,
}

impl CKSyncEngineWillFetchChangesEvent {
    pub fn new(context: CKSyncEngineFetchChangesContext) -> Self {
        Self { context }
    }

    pub const fn context(&self) -> &CKSyncEngineFetchChangesContext {
        &self.context
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CKSyncEngineWillFetchRecordZoneChangesEvent {
    zone_id: CKRecordZoneID,
}

impl CKSyncEngineWillFetchRecordZoneChangesEvent {
    pub fn new(zone_id: CKRecordZoneID) -> Self {
        Self { zone_id }
    }

    pub const fn zone_id(&self) -> &CKRecordZoneID {
        &self.zone_id
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineDidFetchRecordZoneChangesEvent {
    zone_id: CKRecordZoneID,
    error: Option<CloudKitError>,
}

impl CKSyncEngineDidFetchRecordZoneChangesEvent {
    pub fn new(zone_id: CKRecordZoneID) -> Self {
        Self {
            zone_id,
            error: None,
        }
    }

    pub const fn zone_id(&self) -> &CKRecordZoneID {
        &self.zone_id
    }

    pub const fn error(&self) -> Option<&CloudKitError> {
        self.error.as_ref()
    }

    pub fn with_error(mut self, error: CloudKitError) -> Self {
        self.error = Some(error);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineDidFetchChangesEvent {
    context: CKSyncEngineFetchChangesContext,
}

impl CKSyncEngineDidFetchChangesEvent {
    pub fn new(context: CKSyncEngineFetchChangesContext) -> Self {
        Self { context }
    }

    pub const fn context(&self) -> &CKSyncEngineFetchChangesContext {
        &self.context
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineWillSendChangesEvent {
    context: CKSyncEngineSendChangesContext,
}

impl CKSyncEngineWillSendChangesEvent {
    pub fn new(context: CKSyncEngineSendChangesContext) -> Self {
        Self { context }
    }

    pub const fn context(&self) -> &CKSyncEngineSendChangesContext {
        &self.context
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineDidSendChangesEvent {
    context: CKSyncEngineSendChangesContext,
}

impl CKSyncEngineDidSendChangesEvent {
    pub fn new(context: CKSyncEngineSendChangesContext) -> Self {
        Self { context }
    }

    pub const fn context(&self) -> &CKSyncEngineSendChangesContext {
        &self.context
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
enum CKSyncEngineEventKind {
    StateUpdate(CKSyncEngineStateUpdateEvent),
    AccountChange(CKSyncEngineAccountChangeEvent),
    FetchedDatabaseChanges(CKSyncEngineFetchedDatabaseChangesEvent),
    FetchedRecordZoneChanges(CKSyncEngineFetchedRecordZoneChangesEvent),
    SentDatabaseChanges(CKSyncEngineSentDatabaseChangesEvent),
    SentRecordZoneChanges(CKSyncEngineSentRecordZoneChangesEvent),
    WillFetchChanges(CKSyncEngineWillFetchChangesEvent),
    WillFetchRecordZoneChanges(CKSyncEngineWillFetchRecordZoneChangesEvent),
    DidFetchRecordZoneChanges(CKSyncEngineDidFetchRecordZoneChangesEvent),
    DidFetchChanges(CKSyncEngineDidFetchChangesEvent),
    WillSendChanges(CKSyncEngineWillSendChangesEvent),
    DidSendChanges(CKSyncEngineDidSendChangesEvent),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CKSyncEngineEvent {
    kind: CKSyncEngineEventKind,
}

impl CKSyncEngineEvent {
    pub fn event_type(&self) -> CKSyncEngineEventType {
        match &self.kind {
            CKSyncEngineEventKind::StateUpdate(_) => CKSyncEngineEventType::StateUpdate,
            CKSyncEngineEventKind::AccountChange(_) => CKSyncEngineEventType::AccountChange,
            CKSyncEngineEventKind::FetchedDatabaseChanges(_) => {
                CKSyncEngineEventType::FetchedDatabaseChanges
            }
            CKSyncEngineEventKind::FetchedRecordZoneChanges(_) => {
                CKSyncEngineEventType::FetchedRecordZoneChanges
            }
            CKSyncEngineEventKind::SentDatabaseChanges(_) => {
                CKSyncEngineEventType::SentDatabaseChanges
            }
            CKSyncEngineEventKind::SentRecordZoneChanges(_) => {
                CKSyncEngineEventType::SentRecordZoneChanges
            }
            CKSyncEngineEventKind::WillFetchChanges(_) => CKSyncEngineEventType::WillFetchChanges,
            CKSyncEngineEventKind::WillFetchRecordZoneChanges(_) => {
                CKSyncEngineEventType::WillFetchRecordZoneChanges
            }
            CKSyncEngineEventKind::DidFetchRecordZoneChanges(_) => {
                CKSyncEngineEventType::DidFetchRecordZoneChanges
            }
            CKSyncEngineEventKind::DidFetchChanges(_) => CKSyncEngineEventType::DidFetchChanges,
            CKSyncEngineEventKind::WillSendChanges(_) => CKSyncEngineEventType::WillSendChanges,
            CKSyncEngineEventKind::DidSendChanges(_) => CKSyncEngineEventType::DidSendChanges,
        }
    }

    pub fn as_state_update(&self) -> Option<&CKSyncEngineStateUpdateEvent> {
        match &self.kind {
            CKSyncEngineEventKind::StateUpdate(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_account_change(&self) -> Option<&CKSyncEngineAccountChangeEvent> {
        match &self.kind {
            CKSyncEngineEventKind::AccountChange(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_fetched_database_changes(&self) -> Option<&CKSyncEngineFetchedDatabaseChangesEvent> {
        match &self.kind {
            CKSyncEngineEventKind::FetchedDatabaseChanges(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_fetched_record_zone_changes(
        &self,
    ) -> Option<&CKSyncEngineFetchedRecordZoneChangesEvent> {
        match &self.kind {
            CKSyncEngineEventKind::FetchedRecordZoneChanges(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_sent_database_changes(&self) -> Option<&CKSyncEngineSentDatabaseChangesEvent> {
        match &self.kind {
            CKSyncEngineEventKind::SentDatabaseChanges(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_sent_record_zone_changes(&self) -> Option<&CKSyncEngineSentRecordZoneChangesEvent> {
        match &self.kind {
            CKSyncEngineEventKind::SentRecordZoneChanges(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_will_fetch_changes(&self) -> Option<&CKSyncEngineWillFetchChangesEvent> {
        match &self.kind {
            CKSyncEngineEventKind::WillFetchChanges(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_will_fetch_record_zone_changes(
        &self,
    ) -> Option<&CKSyncEngineWillFetchRecordZoneChangesEvent> {
        match &self.kind {
            CKSyncEngineEventKind::WillFetchRecordZoneChanges(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_did_fetch_record_zone_changes(
        &self,
    ) -> Option<&CKSyncEngineDidFetchRecordZoneChangesEvent> {
        match &self.kind {
            CKSyncEngineEventKind::DidFetchRecordZoneChanges(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_did_fetch_changes(&self) -> Option<&CKSyncEngineDidFetchChangesEvent> {
        match &self.kind {
            CKSyncEngineEventKind::DidFetchChanges(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_will_send_changes(&self) -> Option<&CKSyncEngineWillSendChangesEvent> {
        match &self.kind {
            CKSyncEngineEventKind::WillSendChanges(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_did_send_changes(&self) -> Option<&CKSyncEngineDidSendChangesEvent> {
        match &self.kind {
            CKSyncEngineEventKind::DidSendChanges(event) => Some(event),
            _ => None,
        }
    }
}

macro_rules! impl_event_from {
    ($event:ident, $variant:ident) => {
        impl From<$event> for CKSyncEngineEvent {
            fn from(event: $event) -> Self {
                Self {
                    kind: CKSyncEngineEventKind::$variant(event),
                }
            }
        }
    };
}

impl_event_from!(CKSyncEngineStateUpdateEvent, StateUpdate);
impl_event_from!(CKSyncEngineAccountChangeEvent, AccountChange);
impl_event_from!(
    CKSyncEngineFetchedDatabaseChangesEvent,
    FetchedDatabaseChanges
);
impl_event_from!(
    CKSyncEngineFetchedRecordZoneChangesEvent,
    FetchedRecordZoneChanges
);
impl_event_from!(CKSyncEngineSentDatabaseChangesEvent, SentDatabaseChanges);
impl_event_from!(
    CKSyncEngineSentRecordZoneChangesEvent,
    SentRecordZoneChanges
);
impl_event_from!(CKSyncEngineWillFetchChangesEvent, WillFetchChanges);
impl_event_from!(
    CKSyncEngineWillFetchRecordZoneChangesEvent,
    WillFetchRecordZoneChanges
);
impl_event_from!(
    CKSyncEngineDidFetchRecordZoneChangesEvent,
    DidFetchRecordZoneChanges
);
impl_event_from!(CKSyncEngineDidFetchChangesEvent, DidFetchChanges);
impl_event_from!(CKSyncEngineWillSendChangesEvent, WillSendChanges);
impl_event_from!(CKSyncEngineDidSendChangesEvent, DidSendChanges);

#[derive(Clone)]
pub struct CKSyncEngine {
    configuration: CKSyncEngineConfiguration,
    state: CKSyncEngineState,
}

impl fmt::Debug for CKSyncEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CKSyncEngine")
            .field("configuration", &self.configuration)
            .field("state", &self.state)
            .finish()
    }
}

impl CKSyncEngine {
    pub fn new(configuration: CKSyncEngineConfiguration) -> Self {
        Self {
            configuration,
            state: CKSyncEngineState::default(),
        }
    }

    pub const fn configuration(&self) -> &CKSyncEngineConfiguration {
        &self.configuration
    }

    pub const fn database(&self) -> &CKDatabase {
        self.configuration.database()
    }

    pub const fn state(&self) -> &CKSyncEngineState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut CKSyncEngineState {
        &mut self.state
    }

    pub fn fetch_changes(&self, reason: CKSyncEngineSyncReason) -> CKSyncEngineFetchChangesContext {
        let default_options =
            CKSyncEngineFetchChangesOptions::new(CKSyncEngineFetchChangesScope::default());
        let provisional_context =
            CKSyncEngineFetchChangesContext::new(reason, default_options.clone());
        let options = self
            .configuration
            .delegate()
            .next_fetch_changes_options_for_context(self, &provisional_context)
            .unwrap_or(default_options);
        let context = CKSyncEngineFetchChangesContext::new(reason, options);
        self.handle_event(CKSyncEngineWillFetchChangesEvent::new(context.clone()));
        context
    }

    pub fn send_changes(
        &self,
        reason: CKSyncEngineSyncReason,
    ) -> Option<CKSyncEngineRecordZoneChangeBatch> {
        let context = CKSyncEngineSendChangesContext::new(
            reason,
            CKSyncEngineSendChangesOptions::new(CKSyncEngineSendChangesScope::default()),
        );
        self.handle_event(CKSyncEngineWillSendChangesEvent::new(context.clone()));
        self.configuration
            .delegate()
            .next_record_zone_change_batch_for_context(self, &context)
    }

    pub fn cancel_operations(&self) {}

    pub fn handle_event<E>(&self, event: E)
    where
        E: Into<CKSyncEngineEvent>,
    {
        let event = event.into();
        self.configuration.delegate().handle_event(self, &event);
    }
}
