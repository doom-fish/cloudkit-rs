#![allow(
    clippy::redundant_clone,
    clippy::significant_drop_tightening,
    clippy::too_many_lines
)]

use std::sync::{Arc, Mutex};

use cloudkit::prelude::*;

fn sample_container() -> CKContainer {
    CKContainer::container("iCloud.example.expanded-surface")
}

fn sample_zone_id() -> CKRecordZoneID {
    CKRecordZoneID::new("ExpandedSurfaceZone", CK_CURRENT_USER_DEFAULT_NAME)
}

fn sample_record_id() -> CKRecordID {
    CKRecordID::with_zone("expanded-surface-record", sample_zone_id())
}

fn sample_identity() -> CKUserIdentity {
    CKUserIdentity::new()
        .with_user_record_id(sample_record_id())
        .with_lookup_info(CKUserIdentityLookupInfo::with_email_address(
            "tester@example.com",
        ))
        .with_name_components(CKPersonNameComponents::new().with_given_name("Test"))
        .with_has_i_cloud_account(true)
        .with_contact_identifiers(vec!["tester@example.com".into()])
}

fn sample_share_metadata() -> Result<CKShareMetadata, CloudKitError> {
    let mut root_record = CKRecord::new("ExpandedSurfaceTask")?;
    root_record.set_object_for_key("title", "ship v0.2.1");

    let share = CKShare::new_root_record(&root_record)?;
    let root_record_id = root_record.record_id().clone();

    Ok(
        CKShareMetadata::new("iCloud.example.expanded-surface", share, sample_identity())
            .with_hierarchical_root_record_id(root_record_id.clone())
            .with_participant_role(CKShareParticipantRole::Owner)
            .with_participant_status(CKShareParticipantAcceptanceStatus::Accepted)
            .with_participant_permission(CKShareParticipantPermission::ReadWrite)
            .with_root_record(root_record)
            .with_participant_type(CKShareParticipantType::Owner)
            .with_root_record_id(root_record_id),
    )
}

#[test]
fn exported_constants_match_the_sdk_values() {
    assert_eq!(
        CK_ACCOUNT_CHANGED_NOTIFICATION,
        "CKAccountChangedNotification"
    );
    assert_eq!(CK_CURRENT_USER_DEFAULT_NAME, "__defaultOwner__");
    assert_eq!(CK_ERROR_RETRY_AFTER_KEY, "CKRetryAfter");
    assert_eq!(
        CK_ERROR_USER_DID_RESET_ENCRYPTED_DATA_KEY,
        "CKUserDidResetEncryptedData"
    );
    assert_eq!(CK_PARTIAL_ERRORS_BY_ITEM_ID_KEY, "CKPartialErrors");
    assert_eq!(
        CK_RECORD_CHANGED_ERROR_ANCESTOR_RECORD_KEY,
        "AncestorRecord"
    );
    assert_eq!(CK_RECORD_CHANGED_ERROR_CLIENT_RECORD_KEY, "ClientRecord");
    assert_eq!(CK_RECORD_CHANGED_ERROR_SERVER_RECORD_KEY, "ServerRecord");
    assert_eq!(CK_QUERY_OPERATION_MAXIMUM_RESULTS, 0);
    assert_eq!(CK_RECORD_CREATION_DATE_KEY, "___createTime");
    assert_eq!(CK_RECORD_CREATOR_USER_RECORD_ID_KEY, "___createdBy");
    assert_eq!(CK_RECORD_LAST_MODIFIED_USER_RECORD_ID_KEY, "___modifiedBy");
    assert_eq!(CK_RECORD_MODIFICATION_DATE_KEY, "___modTime");
    assert_eq!(CK_RECORD_PARENT_KEY, "___parent");
    assert_eq!(CK_RECORD_RECORD_ID_KEY, "___recordID");
    assert_eq!(CK_RECORD_SHARE_KEY, "___share");
    assert_eq!(CK_RECORD_TYPE_USER_RECORD, "Users");
    assert_eq!(CK_RECORD_ZONE_DEFAULT_NAME, "_defaultZone");
    assert_eq!(CK_RECORD_NAME_ZONE_WIDE_SHARE, "cloudkit.zoneshare");
    assert_eq!(CK_RECORD_TYPE_SHARE, "cloudkit.share");
    assert_eq!(
        CK_SHARE_THUMBNAIL_IMAGE_DATA_KEY,
        "cloudkit.thumbnailImageData"
    );
    assert_eq!(CK_SHARE_TITLE_KEY, "cloudkit.title");
    assert_eq!(CK_SHARE_TYPE_KEY, "cloudkit.type");
}

#[test]
fn notification_and_share_support_types_capture_local_state(
) -> Result<(), Box<dyn std::error::Error>> {
    let zone_id = sample_zone_id();
    let record_id = sample_record_id();
    let identity = sample_identity();
    let lookup_info = CKUserIdentityLookupInfo::with_email_address("tester@example.com");

    let notification = CKNotification::new(CKNotificationType::Query)
        .with_notification_id(CKNotificationID::new(vec![1, 2, 3]))
        .with_container_identifier("iCloud.example.expanded-surface")
        .with_subscription_owner_user_record_id(record_id.clone())
        .with_pruned(true)
        .with_subscription_id("subscription-id");
    let query_notification = CKQueryNotification::new(
        CKQueryNotificationReason::RecordUpdated,
        CKDatabaseScope::Private,
    )
    .with_notification(notification.clone())
    .with_record_id(record_id.clone())
    .with_record_field("title", "updated");
    let zone_notification = CKRecordZoneNotification::new(CKDatabaseScope::Shared)
        .with_notification(CKNotification::new(CKNotificationType::RecordZone))
        .with_record_zone_id(zone_id.clone());
    let database_notification = CKDatabaseNotification::new(CKDatabaseScope::Public)
        .with_notification(CKNotification::new(CKNotificationType::Database));

    let access_options = CKSharingParticipantAccessOption::ANYONE_WITH_LINK
        | CKSharingParticipantAccessOption::SPECIFIED_RECIPIENTS_ONLY;
    let permission_options = CKSharingParticipantPermissionOption::READ_ONLY
        | CKSharingParticipantPermissionOption::READ_WRITE;
    let sharing_options = CKAllowedSharingOptions::new(permission_options, access_options);
    let requester = CKShareAccessRequester::new(identity.clone(), lookup_info.clone())
        .with_contact_display_name("Tester");
    let blocked_identity =
        CKShareBlockedIdentity::new(identity.clone()).with_contact_display_name("Blocked Tester");

    let metadata = sample_share_metadata()?;
    let saved_calls = Arc::new(Mutex::new(0_u32));
    let stopped_calls = Arc::new(Mutex::new(0_u32));
    let saved_calls_for_handler = Arc::clone(&saved_calls);
    let stopped_calls_for_handler = Arc::clone(&stopped_calls);
    let observer = CKSystemSharingUIObserver::new(sample_container())
        .with_did_save_share_handler(move |saved_record_id, share, error| {
            assert_eq!(saved_record_id.record_name(), "expanded-surface-record");
            assert!(share.is_some());
            assert!(error.is_none());
            *saved_calls_for_handler.lock().expect("save handler mutex") += 1;
        })
        .with_did_stop_sharing_handler(move |stopped_record_id, error| {
            assert_eq!(stopped_record_id.record_name(), "expanded-surface-record");
            assert!(error.is_none());
            *stopped_calls_for_handler
                .lock()
                .expect("stop handler mutex") += 1;
        });

    observer.notify_did_save_share(&record_id, Some(metadata.share()), None);
    observer.notify_did_stop_sharing(&record_id, None);

    assert_eq!(notification.notification_type(), CKNotificationType::Query);
    assert_eq!(
        query_notification.query_notification_reason(),
        CKQueryNotificationReason::RecordUpdated
    );
    assert_eq!(query_notification.record_fields().len(), 1);
    assert_eq!(zone_notification.database_scope(), CKDatabaseScope::Shared);
    assert_eq!(
        database_notification.database_scope(),
        CKDatabaseScope::Public
    );
    assert!(sharing_options
        .allowed_participant_access_options()
        .contains(CKSharingParticipantAccessOption::ANYONE_WITH_LINK));
    assert!(sharing_options
        .allowed_participant_permission_options()
        .contains(CKSharingParticipantPermissionOption::READ_WRITE));
    assert_eq!(requester.contact_display_name(), Some("Tester"));
    assert_eq!(
        blocked_identity.contact_display_name(),
        Some("Blocked Tester")
    );
    assert_eq!(
        metadata.container_identifier(),
        "iCloud.example.expanded-surface"
    );
    assert_eq!(
        metadata.participant_type(),
        Some(CKShareParticipantType::Owner)
    );
    assert_eq!(
        metadata.participant_status(),
        CKShareParticipantAcceptanceStatus::Accepted
    );
    assert_eq!(
        metadata.participant_permission(),
        CKShareParticipantPermission::ReadWrite
    );
    assert_eq!(
        metadata
            .share()
            .owner()
            .expect("share owner")
            .participant_type(),
        CKShareParticipantType::Owner
    );
    assert_eq!(*saved_calls.lock().expect("save count mutex"), 1);
    assert_eq!(*stopped_calls.lock().expect("stop count mutex"), 1);

    Ok(())
}

#[test]
fn operation_support_types_store_requested_configuration() -> Result<(), Box<dyn std::error::Error>>
{
    let container = sample_container();
    let database = container.private_cloud_database();
    let zone_id = sample_zone_id();
    let zone = CKRecordZone::new(zone_id.zone_name());
    let share_metadata = sample_share_metadata()?;

    let configuration = CKOperationConfiguration::new()
        .with_container(container.clone())
        .with_quality_of_service(CKQualityOfService::Utility)
        .with_allows_cellular_access(false)
        .with_long_lived(true)
        .with_timeout_interval_for_request(5.0)
        .with_timeout_interval_for_resource(10.0);
    let group = CKOperationGroup::new()
        .with_name("expanded-surface")
        .with_quantity(3)
        .with_expected_send_size(CKOperationGroupTransferSize::Kilobytes)
        .with_expected_receive_size(CKOperationGroupTransferSize::Megabytes);
    let mut operation = CKOperation::new()
        .with_configuration(configuration.clone())
        .with_group(group.clone());
    operation.mark_long_lived_operation_was_persisted();

    let database_operation = CKDatabaseOperation::new()
        .with_operation(operation.clone())
        .with_database(database.clone());
    let fetch_zones = CKFetchRecordZonesOperation::new(vec![zone_id.clone()])
        .with_database_operation(database_operation.clone());
    let modify_zones = CKModifyRecordZonesOperation::new(vec![zone.clone()], vec![zone_id.clone()])
        .with_database_operation(database_operation.clone());
    let fetch_subscriptions = CKFetchSubscriptionsOperation::new(vec!["sub-id".into()])
        .with_database_operation(database_operation.clone());
    let modify_subscriptions = CKModifySubscriptionsOperation::new(
        vec![CKDatabaseSubscription::new("sub-id").into()],
        vec!["old-sub-id".into()],
    )
    .with_database_operation(database_operation.clone());
    let fetch_share_participants = CKFetchShareParticipantsOperation::new()
        .with_operation(operation.clone())
        .with_user_identity_lookup_infos(vec![CKUserIdentityLookupInfo::with_email_address(
            "tester@example.com",
        )]);
    let fetch_share_metadata = CKFetchShareMetadataOperation::new()
        .with_operation(operation.clone())
        .with_share_urls(vec!["https://example.com/share".into()])
        .with_should_fetch_root_record(true)
        .with_root_record_desired_keys(vec!["title".into()]);
    let accept_shares = CKAcceptSharesOperation::new()
        .with_operation(operation.clone())
        .with_share_metadatas(vec![share_metadata]);
    let request_share_access = CKShareRequestAccessOperation::new()
        .with_operation(operation.clone())
        .with_share_urls(vec!["https://example.com/share".into()]);
    let location_sort_descriptor = CKLocationSortDescriptor::new("location", 37.33, -122.03);

    let missing_api_token = CKFetchWebAuthTokenOperation::new()
        .with_database_operation(database_operation)
        .execute_in(&database)
        .expect_err("missing API token should fail before reaching CloudKit");

    assert_eq!(
        configuration.quality_of_service(),
        CKQualityOfService::Utility
    );
    assert!(!configuration.allows_cellular_access());
    assert!(configuration.long_lived());
    assert_eq!(group.name(), Some("expanded-surface"));
    assert!(operation.long_lived_operation_was_persisted());
    assert_eq!(fetch_zones.record_zone_ids().expect("zone ids").len(), 1);
    assert_eq!(modify_zones.record_zones_to_save().len(), 1);
    assert_eq!(
        fetch_subscriptions
            .subscription_ids()
            .expect("subscription ids")
            .len(),
        1
    );
    assert_eq!(modify_subscriptions.subscriptions_to_save().len(), 1);
    assert_eq!(
        fetch_share_participants.user_identity_lookup_infos().len(),
        1
    );
    assert!(fetch_share_metadata.should_fetch_root_record());
    assert_eq!(accept_shares.share_metadatas().len(), 1);
    assert_eq!(request_share_access.share_urls().len(), 1);
    assert_eq!(location_sort_descriptor.key(), "location");
    assert_eq!(
        missing_api_token.kind(),
        CloudKitErrorCode::BridgeInvalidArgument
    );

    Ok(())
}

#[derive(Default)]
struct RecordingSyncDelegate {
    event_types: Mutex<Vec<CKSyncEngineEventType>>,
}

impl CKSyncEngineDelegate for RecordingSyncDelegate {
    fn handle_event(&self, _engine: &CKSyncEngine, event: &CKSyncEngineEvent) {
        self.event_types
            .lock()
            .expect("event type mutex")
            .push(event.event_type());
    }

    fn next_record_zone_change_batch_for_context(
        &self,
        _engine: &CKSyncEngine,
        _context: &CKSyncEngineSendChangesContext,
    ) -> Option<CKSyncEngineRecordZoneChangeBatch> {
        Some(CKSyncEngineRecordZoneChangeBatch::from_pending_changes(
            vec![CKSyncEnginePendingRecordZoneChange::new(
                sample_record_id(),
                CKSyncEnginePendingRecordZoneChangeType::DeleteRecord,
            )],
            Vec::new(),
            vec![sample_record_id()],
        ))
    }

    fn next_fetch_changes_options_for_context(
        &self,
        _engine: &CKSyncEngine,
        _context: &CKSyncEngineFetchChangesContext,
    ) -> Option<CKSyncEngineFetchChangesOptions> {
        Some(
            CKSyncEngineFetchChangesOptions::new(
                CKSyncEngineFetchChangesScope::new().add_zone_id(sample_zone_id()),
            )
            .with_prioritized_zone_id(sample_zone_id()),
        )
    }
}

#[test]
fn sync_engine_surface_builds_state_batches_and_events() -> Result<(), Box<dyn std::error::Error>> {
    let container = sample_container();
    let database = container.private_cloud_database();
    let zone_id = sample_zone_id();
    let record_id = sample_record_id();
    let record_zone = CKRecordZone::new(zone_id.zone_name());
    let record = CKRecord::new("ExpandedSyncRecord")?;
    let state_serialization = CKSyncEngineStateSerialization::new(vec![9, 8, 7]);

    let fetch_scope = CKSyncEngineFetchChangesScope::new()
        .add_zone_id(zone_id.clone())
        .add_excluded_zone_id(CKRecordZoneID::new(
            "ExcludedZone",
            CK_CURRENT_USER_DEFAULT_NAME,
        ));
    let send_scope = CKSyncEngineSendChangesScope::new()
        .add_zone_id(zone_id.clone())
        .add_record_id(record_id.clone());
    let group = CKOperationGroup::new().with_name("sync-group");
    let fetch_options = CKSyncEngineFetchChangesOptions::new(fetch_scope.clone())
        .with_operation_group(group.clone())
        .with_prioritized_zone_id(zone_id.clone());
    let send_options =
        CKSyncEngineSendChangesOptions::new(send_scope.clone()).with_operation_group(group.clone());
    let fetch_context =
        CKSyncEngineFetchChangesContext::new(CKSyncEngineSyncReason::Manual, fetch_options.clone());
    let send_context = CKSyncEngineSendChangesContext::new(
        CKSyncEngineSyncReason::Scheduled,
        send_options.clone(),
    );

    let pending_record_save = CKSyncEnginePendingRecordZoneChange::new(
        record_id.clone(),
        CKSyncEnginePendingRecordZoneChangeType::SaveRecord,
    );
    let pending_record_delete = CKSyncEnginePendingRecordZoneChange::new(
        record_id.clone(),
        CKSyncEnginePendingRecordZoneChangeType::DeleteRecord,
    );
    let pending_zone_save = CKSyncEnginePendingZoneSave::new(record_zone.clone());
    let pending_zone_delete = CKSyncEnginePendingZoneDelete::new(zone_id.clone());

    let mut state = CKSyncEngineState::new();
    state.add_pending_record_zone_change(pending_record_save.clone());
    state.add_pending_record_zone_change(pending_record_delete.clone());
    state.add_pending_database_change(pending_zone_save.clone().into_pending_change());
    state.add_pending_database_change(pending_zone_delete.clone().into_pending_change());
    state.set_has_pending_untracked_changes(true);
    state.add_zone_id_with_unfetched_server_changes(zone_id.clone());

    let batch =
        CKSyncEngineRecordZoneChangeBatch::new(vec![record.clone()], vec![record_id.clone()], true)
            .with_atomic_by_zone(false);

    let missing_api_token = CKFetchWebAuthTokenOperation::new()
        .execute_in(&database)
        .expect_err("missing API token should fail before reaching CloudKit");
    let fetched_record_deletion =
        CKSyncEngineFetchedRecordDeletion::new(record_id.clone(), "ExpandedSyncRecord");
    let fetched_zone_deletion = CKSyncEngineFetchedZoneDeletion::new(
        zone_id.clone(),
        CKSyncEngineZoneDeletionReason::Deleted,
    );
    let failed_record_save =
        CKSyncEngineFailedRecordSave::new(record.clone(), missing_api_token.clone());
    let failed_zone_save =
        CKSyncEngineFailedZoneSave::new(record_zone.clone(), missing_api_token.clone());
    let failed_record_delete =
        CKSyncEngineFailedRecordDelete::new(record_id.clone(), missing_api_token.clone());
    let failed_zone_delete =
        CKSyncEngineFailedZoneDelete::new(zone_id.clone(), missing_api_token.clone());

    let state_update = CKSyncEngineStateUpdateEvent::new(state_serialization.clone());
    let account_change = CKSyncEngineAccountChangeEvent::new(CKSyncEngineAccountChangeType::SignIn)
        .with_previous_user(sample_identity())
        .with_current_user(sample_identity());
    let fetched_db_changes = CKSyncEngineFetchedDatabaseChangesEvent::new(
        vec![record_zone.clone()],
        vec![fetched_zone_deletion.clone()],
    );
    let fetched_record_zone_changes = CKSyncEngineFetchedRecordZoneChangesEvent::new(
        vec![record.clone()],
        vec![fetched_record_deletion.clone()],
    );
    let sent_db_changes = CKSyncEngineSentDatabaseChangesEvent::new(
        vec![record_zone.clone()],
        vec![failed_zone_save.clone()],
        vec![zone_id.clone()],
        vec![failed_zone_delete.clone()],
    );
    let sent_record_zone_changes = CKSyncEngineSentRecordZoneChangesEvent::new(
        vec![record.clone()],
        vec![failed_record_save.clone()],
        vec![record_id.clone()],
        vec![failed_record_delete.clone()],
    );
    let will_fetch = CKSyncEngineWillFetchChangesEvent::new(fetch_context.clone());
    let will_fetch_zone = CKSyncEngineWillFetchRecordZoneChangesEvent::new(zone_id.clone());
    let did_fetch_zone = CKSyncEngineDidFetchRecordZoneChangesEvent::new(zone_id.clone())
        .with_error(missing_api_token.clone());
    let did_fetch = CKSyncEngineDidFetchChangesEvent::new(fetch_context.clone());
    let will_send = CKSyncEngineWillSendChangesEvent::new(send_context.clone());
    let did_send = CKSyncEngineDidSendChangesEvent::new(send_context.clone());

    let delegate = Arc::new(RecordingSyncDelegate::default());
    let delegate_for_config: Arc<dyn CKSyncEngineDelegate> = delegate.clone();
    let configuration = CKSyncEngineConfiguration::new(
        database,
        Some(state_serialization.clone()),
        delegate_for_config,
    )
    .with_automatically_sync(false)
    .with_subscription_id("subscription-id");
    let engine = CKSyncEngine::new(configuration);
    let runtime_fetch_context = engine.fetch_changes(CKSyncEngineSyncReason::Manual);
    let runtime_batch = engine
        .send_changes(CKSyncEngineSyncReason::Scheduled)
        .expect("delegate should supply a batch");

    engine.handle_event(state_update.clone());
    engine.handle_event(account_change.clone());
    engine.handle_event(fetched_db_changes.clone());
    engine.handle_event(fetched_record_zone_changes.clone());
    engine.handle_event(sent_db_changes.clone());
    engine.handle_event(sent_record_zone_changes.clone());
    engine.handle_event(will_fetch.clone());
    engine.handle_event(will_fetch_zone.clone());
    engine.handle_event(did_fetch_zone.clone());
    engine.handle_event(did_fetch.clone());
    engine.handle_event(will_send.clone());
    engine.handle_event(did_send.clone());

    let event: CKSyncEngineEvent = will_fetch.clone().into();
    let recorded_event_types = delegate.event_types.lock().expect("delegate event mutex");

    assert_eq!(fetch_scope.zone_ids().len(), 1);
    assert_eq!(send_scope.record_ids().len(), 1);
    assert_eq!(fetch_options.prioritized_zone_ids().len(), 1);
    assert!(send_options.scope().contains_record_id(&record_id));
    assert_eq!(state.pending_record_zone_changes().len(), 2);
    assert_eq!(state.pending_database_changes().len(), 2);
    assert!(state.has_pending_untracked_changes());
    assert_eq!(state.zone_ids_with_unfetched_server_changes().len(), 1);
    assert!(!batch.atomic_by_zone());
    assert_eq!(batch.record_ids_to_delete().len(), 1);
    assert_eq!(fetched_record_deletion.record_type(), "ExpandedSyncRecord");
    assert_eq!(
        fetched_zone_deletion.reason(),
        CKSyncEngineZoneDeletionReason::Deleted
    );
    assert_eq!(
        failed_record_save.record().record_type(),
        "ExpandedSyncRecord"
    );
    assert_eq!(
        failed_zone_save.record_zone().zone_id().zone_name(),
        zone_id.zone_name()
    );
    assert_eq!(
        state_update.state_serialization().archived_data(),
        &[9, 8, 7]
    );
    assert_eq!(
        account_change.change_type(),
        CKSyncEngineAccountChangeType::SignIn
    );
    assert_eq!(fetched_db_changes.modifications().len(), 1);
    assert_eq!(fetched_record_zone_changes.modifications().len(), 1);
    assert_eq!(sent_db_changes.failed_zone_saves().len(), 1);
    assert_eq!(sent_record_zone_changes.failed_record_saves().len(), 1);
    assert_eq!(event.event_type(), CKSyncEngineEventType::WillFetchChanges);
    assert_eq!(
        runtime_fetch_context.reason(),
        CKSyncEngineSyncReason::Manual
    );
    assert_eq!(runtime_batch.record_ids_to_delete().len(), 1);
    assert!(recorded_event_types.contains(&CKSyncEngineEventType::WillFetchChanges));
    assert!(recorded_event_types.contains(&CKSyncEngineEventType::WillSendChanges));
    assert!(recorded_event_types.contains(&CKSyncEngineEventType::StateUpdate));

    Ok(())
}
