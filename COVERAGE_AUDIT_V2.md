# cloudkit-rs coverage audit v2 (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 140
VERIFIED: 77
MODELED: 57
GAPS: 0
EXEMPT: 6
COVERAGE_PCT: 57.46%

This audit lists `CloudKit.framework` symbols from the MacOSX26.2.sdk header set; it was not regenerated against the installed 26.5 SDK. A symbol counts as VERIFIED only when the crate reaches the real framework object through the Swift bridge. MODELED symbols have a Rust type, but that type is a local model or simulation that never touches CloudKit. `COVERAGE_PCT` is VERIFIED divided by the 134 non-exempt symbols; it does not measure how much of each class's API is bridged.

The audit was conducted by systematically enumerating all public symbols in the CloudKit.framework headers and cross-referencing with the crate's Rust wrapper (src/*.rs) and Swift bridge (swift-bridge/Sources/**/*.swift). The v1 audit was re-verified against MacOSX26.2.sdk headers; all EXEMPT promotions were confirmed valid (deprecated macOS symbols with API_DEPRECATED attributes). No new symbols were added in 26.2 SDK relative to the previous version.

## �� VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| `CKAsset` | class | `CKAsset.h` | `CKAsset` (`src/record.rs:160`) |
| `CKAccountStatus` | enum | `CKContainer.h` | AccountStatus (`src/container.rs:17`) |
| `CKContainer` | class | `CKContainer.h` | `CKContainer` (`src/container.rs:54`) |
| `CKDatabase` | class | `CKDatabase.h` | `CKDatabase` (`src/database.rs:24`) |
| `CKDatabaseScope` | enum | `CKDatabase.h` | `CKDatabaseScope` (`src/database.rs:17`) |
| `CKErrorCode` | enum | `CKError.h` | CloudKitErrorCode (`src/error.rs:10`) |
| `CKErrorDomain` | constant | `CKError.h` | CLOUDKIT_ERROR_DOMAIN (`src/error.rs:5`) |
| `CKFetchDatabaseChangesOperation` | class | `CKFetchDatabaseChangesOperation.h` | `CKFetchDatabaseChangesOperation` (`src/operation.rs:350`) |
| `CKFetchRecordZoneChangesConfiguration` | class | `CKFetchRecordZoneChangesOperation.h` | `CKFetchRecordZoneChangesConfiguration` (`src/operation.rs:437`) |
| `CKFetchRecordZoneChangesOperation` | class | `CKFetchRecordZoneChangesOperation.h` | `CKFetchRecordZoneChangesOperation` (`src/operation.rs:498`) |
| `CKFetchRecordsOperation` | class | `CKFetchRecordsOperation.h` | `CKFetchRecordsOperation` (`src/operation.rs:284`) |
| `CKModifyRecordsOperation` | class | `CKModifyRecordsOperation.h` | `CKModifyRecordsOperation` (`src/operation.rs:67`) |
| `CKRecordSavePolicy` | enum | `CKModifyRecordsOperation.h` | `CKRecordSavePolicy` (`src/operation.rs:23`) |
| `CKQuery` | class | `CKQuery.h` | `CKQuery` (`src/query.rs:34`) |
| `CKQueryCursor` | class | `CKQueryOperation.h` | `CKQueryCursor` (`src/fetched_results.rs:11`) |
| `CKQueryOperation` | class | `CKQueryOperation.h` | `CKQueryOperation` (`src/operation.rs:183`) |
| `CKRecord` | class | `CKRecord.h` | `CKRecord` (`src/record.rs:474`) |
| `CKRecordValue` | protocol | `CKRecord.h` | RecordValue (`src/record.rs:188`) |
| `CKRecordID` | class | `CKRecordID.h` | `CKRecordID` (`src/record.rs:119`) |
| `CKRecordZone` | class | `CKRecordZone.h` | `CKRecordZone` (`src/record.rs:406`) |
| `CKRecordZoneCapabilities` | enum | `CKRecordZone.h` | `CKRecordZoneCapabilities` (`src/record.rs:19`) |
| `CKRecordZoneEncryptionScope` | enum | `CKRecordZone.h` | `CKRecordZoneEncryptionScope` (`src/record.rs:56`) |
| `CKRecordZoneID` | class | `CKRecordZoneID.h` | `CKRecordZoneID` (`src/record.rs:81`) |
| `CKReference` | class | `CKReference.h` | `CKReference` (`src/reference_utility.rs:31`) |
| `CKReferenceAction` | enum | `CKReference.h` | `CKReferenceAction` (`src/reference_utility.rs:6`) |
| `CKServerChangeToken` | class | `CKServerChangeToken.h` | `CKServerChangeToken` (`src/server_change_token.rs:4`) |
| `CKShare` | class | `CKShare.h` | `CKShare` (`src/share.rs:198`) |
| `CKShareParticipant` | class | `CKShareParticipant.h` | `CKShareParticipant` (`src/share.rs:89`) |
| `CKShareParticipantAcceptanceStatus` | enum | `CKShareParticipant.h` | `CKShareParticipantAcceptanceStatus` (`src/share.rs:12`) |
| `CKShareParticipantPermission` | enum | `CKShareParticipant.h` | `CKShareParticipantPermission` (`src/share.rs:34`) |
| `CKShareParticipantRole` | enum | `CKShareParticipant.h` | `CKShareParticipantRole` (`src/share.rs:66`) |
| `CKDatabaseSubscription` | class | `CKSubscription.h` | `CKDatabaseSubscription` (`src/subscription.rs:242`) |
| `CKNotificationInfo` | class | `CKSubscription.h` | `CKNotificationInfo` (`src/notification_info.rs:4`) |
| `CKQuerySubscription` | class | `CKSubscription.h` | `CKQuerySubscription` (`src/subscription.rs:91`) |
| `CKQuerySubscriptionOptions` | enum | `CKSubscription.h` | QuerySubscriptionOptions (`src/subscription.rs:28`) |
| `CKRecordZoneSubscription` | class | `CKSubscription.h` | `CKRecordZoneSubscription` (`src/subscription.rs:176`) |
| `CKSubscription` | class | `CKSubscription.h` | `CKSubscription` (`src/subscription.rs:56`) |
| `CKSubscriptionType` | enum | `CKSubscription.h` | `CKSubscriptionType` (`src/subscription.rs:9`) |
| `CKUserIdentity` | class | `CKUserIdentity.h` | `CKUserIdentity` (`src/user_identity.rs:160`) |
| `CKUserIdentityLookupInfo` | class | `CKUserIdentityLookupInfo.h` | `CKUserIdentityLookupInfo` (`src/user_identity.rs:99`) |
| `CKAcceptSharesOperation` | class | `CKAcceptSharesOperation.h` | `CKAcceptSharesOperation` (`src/operation_support.rs:1003`) |
| `CKAccountChangedNotification` | constant | `CKContainer.h` | `CK_ACCOUNT_CHANGED_NOTIFICATION` (`src/constants.rs:1`) |
| `CKApplicationPermissions` | enum | `CKContainer.h` | `CKApplicationPermissions` (`src/container.rs:54`) |
| `CKCurrentUserDefaultName` | constant | `CKContainer.h` | `CK_CURRENT_USER_DEFAULT_NAME` (`src/constants.rs:2`) |
| `CKErrorRetryAfterKey` | constant | `CKError.h` | `CK_ERROR_RETRY_AFTER_KEY` (`src/constants.rs:4`) |
| `CKErrorUserDidResetEncryptedDataKey` | constant | `CKError.h` | `CK_ERROR_USER_DID_RESET_ENCRYPTED_DATA_KEY` (`src/constants.rs:5`) |
| `CKPartialErrorsByItemIDKey` | constant | `CKError.h` | `CK_PARTIAL_ERRORS_BY_ITEM_ID_KEY` (`src/constants.rs:6`) |
| `CKRecordChangedErrorAncestorRecordKey` | constant | `CKError.h` | `CK_RECORD_CHANGED_ERROR_ANCESTOR_RECORD_KEY` (`src/constants.rs:7`) |
| `CKRecordChangedErrorClientRecordKey` | constant | `CKError.h` | `CK_RECORD_CHANGED_ERROR_CLIENT_RECORD_KEY` (`src/constants.rs:8`) |
| `CKRecordChangedErrorServerRecordKey` | constant | `CKError.h` | `CK_RECORD_CHANGED_ERROR_SERVER_RECORD_KEY` (`src/constants.rs:9`) |
| `CKFetchRecordZonesOperation` | class | `CKFetchRecordZonesOperation.h` | `CKFetchRecordZonesOperation` (`src/operation_support.rs:332`) |
| `CKFetchShareMetadataOperation` | class | `CKFetchShareMetadataOperation.h` | `CKFetchShareMetadataOperation` (`src/operation_support.rs:874`) |
| `CKFetchShareParticipantsOperation` | class | `CKFetchShareParticipantsOperation.h` | `CKFetchShareParticipantsOperation` (`src/operation_support.rs:791`) |
| `CKFetchSubscriptionsOperation` | class | `CKFetchSubscriptionsOperation.h` | `CKFetchSubscriptionsOperation` (`src/operation_support.rs:527`) |
| `CKFetchWebAuthTokenOperation` | class | `CKFetchWebAuthTokenOperation.h` | `CKFetchWebAuthTokenOperation` (`src/operation_support.rs:710`) |
| `CKLocationSortDescriptor` | class | `CKLocationSortDescriptor.h` | `CKLocationSortDescriptor` (`src/query.rs:34`) |
| `CKModifyRecordZonesOperation` | class | `CKModifyRecordZonesOperation.h` | `CKModifyRecordZonesOperation` (`src/operation_support.rs:427`) |
| `CKModifySubscriptionsOperation` | class | `CKModifySubscriptionsOperation.h` | `CKModifySubscriptionsOperation` (`src/operation_support.rs:623`) |
| `CKQueryOperationMaximumResults` | constant | `CKQueryOperation.h` | `CK_QUERY_OPERATION_MAXIMUM_RESULTS` (`src/constants.rs:11`) |
| `CKRecordCreationDateKey` | constant | `CKRecord.h` | `CK_RECORD_CREATION_DATE_KEY` (`src/constants.rs:13`) |
| `CKRecordCreatorUserRecordIDKey` | constant | `CKRecord.h` | `CK_RECORD_CREATOR_USER_RECORD_ID_KEY` (`src/constants.rs:14`) |
| `CKRecordKeyValueSetting` | protocol | `CKRecord.h` | `CKRecordKeyValueSetting` (`src/record.rs:676`) |
| `CKRecordLastModifiedUserRecordIDKey` | constant | `CKRecord.h` | `CK_RECORD_LAST_MODIFIED_USER_RECORD_ID_KEY` (`src/constants.rs:15`) |
| `CKRecordModificationDateKey` | constant | `CKRecord.h` | `CK_RECORD_MODIFICATION_DATE_KEY` (`src/constants.rs:16`) |
| `CKRecordParentKey` | constant | `CKRecord.h` | `CK_RECORD_PARENT_KEY` (`src/constants.rs:18`) |
| `CKRecordRecordIDKey` | constant | `CKRecord.h` | `CK_RECORD_RECORD_ID_KEY` (`src/constants.rs:19`) |
| `CKRecordShareKey` | constant | `CKRecord.h` | `CK_RECORD_SHARE_KEY` (`src/constants.rs:20`) |
| `CKRecordTypeUserRecord` | constant | `CKRecord.h` | `CK_RECORD_TYPE_USER_RECORD` (`src/constants.rs:22`) |
| `CKRecordZoneDefaultName` | constant | `CKRecordZone.h` | `CK_RECORD_ZONE_DEFAULT_NAME` (`src/constants.rs:23`) |
| `CKRecordNameZoneWideShare` | constant | `CKShare.h` | `CK_RECORD_NAME_ZONE_WIDE_SHARE` (`src/constants.rs:17`) |
| `CKRecordTypeShare` | constant | `CKShare.h` | `CK_RECORD_TYPE_SHARE` (`src/constants.rs:21`) |
| `CKShareThumbnailImageDataKey` | constant | `CKShare.h` | `CK_SHARE_THUMBNAIL_IMAGE_DATA_KEY` (`src/constants.rs:24`) |
| `CKShareTitleKey` | constant | `CKShare.h` | `CK_SHARE_TITLE_KEY` (`src/constants.rs:25`) |
| `CKShareTypeKey` | constant | `CKShare.h` | `CK_SHARE_TYPE_KEY` (`src/constants.rs:26`) |
| `CKShareMetadata` | class | `CKShareMetadata.h` | `CKShareMetadata` (`src/share.rs:595`) |
| `CKShareParticipantType` | enum | `CKShareParticipant.h` | `CKShareParticipantType` (`src/share.rs:394`) |
| `CKShareRequestAccessOperation` | class | `CKShareRequestAccessOperation.h` | `CKShareRequestAccessOperation` (`src/operation_support.rs:1097`) |

## 🟡 MODELED (Rust-only)
| Symbol | Kind | Header | Wrapped by | Why it is not VERIFIED |
| --- | --- | --- | --- | --- |
| `CKAllowedSharingOptions` | class | `CKAllowedSharingOptions.h` | `CKAllowedSharingOptions` (`src/share.rs:489`) | Rust value model; never populated from or passed to CloudKit |
| `CKSharingParticipantAccessOption` | enum | `CKAllowedSharingOptions.h` | `CKSharingParticipantAccessOption` (`src/share.rs:427`) | Rust value model; never populated from or passed to CloudKit |
| `CKSharingParticipantPermissionOption` | enum | `CKAllowedSharingOptions.h` | `CKSharingParticipantPermissionOption` (`src/share.rs:458`) | Rust value model; never populated from or passed to CloudKit |
| `CKDatabaseOperation` | class | `CKDatabaseOperation.h` | `CKDatabaseOperation` (`src/operation_support.rs:275`) | Rust builder; its settings are not applied to the CloudKit operations the crate runs |
| `CKDatabaseNotification` | class | `CKNotification.h` | `CKDatabaseNotification` (`src/notification.rs:259`) | Rust value model; nothing parses a real push payload into it |
| `CKNotification` | class | `CKNotification.h` | `CKNotification` (`src/notification.rs:83`) | Rust value model; nothing parses a real push payload into it |
| `CKNotificationID` | class | `CKNotification.h` | `CKNotificationID` (`src/notification.rs:7`) | Rust value model; nothing parses a real push payload into it |
| `CKNotificationType` | enum | `CKNotification.h` | `CKNotificationType` (`src/notification.rs:23`) | Rust value model; nothing parses a real push payload into it |
| `CKQueryNotification` | class | `CKNotification.h` | `CKQueryNotification` (`src/notification.rs:158`) | Rust value model; nothing parses a real push payload into it |
| `CKQueryNotificationReason` | enum | `CKNotification.h` | `CKQueryNotificationReason` (`src/notification.rs:55`) | Rust value model; nothing parses a real push payload into it |
| `CKRecordZoneNotification` | class | `CKNotification.h` | `CKRecordZoneNotification` (`src/notification.rs:220`) | Rust value model; nothing parses a real push payload into it |
| `CKOperation` | class | `CKOperation.h` | `CKOperation` (`src/operation_support.rs:220`) | Rust builder; its settings are not applied to the CloudKit operations the crate runs |
| `CKOperationConfiguration` | class | `CKOperation.h` | `CKOperationConfiguration` (`src/operation_support.rs:35`) | Rust builder; its settings are not applied to the CloudKit operations the crate runs |
| `CKOperationGroup` | class | `CKOperationGroup.h` | `CKOperationGroup` (`src/operation_support.rs:133`) | Rust builder; its settings are not applied to the CloudKit operations the crate runs |
| `CKOperationGroupTransferSize` | enum | `CKOperationGroup.h` | `CKOperationGroupTransferSize` (`src/operation_support.rs:121`) | Rust builder; its settings are not applied to the CloudKit operations the crate runs |
| `CKShareAccessRequester` | class | `CKShareAccessRequester.h` | `CKShareAccessRequester` (`src/share.rs:530`) | Rust value model; never populated from or passed to CloudKit |
| `CKShareBlockedIdentity` | class | `CKShareBlockedIdentity.h` | `CKShareBlockedIdentity` (`src/share.rs:567`) | Rust value model; never populated from or passed to CloudKit |
| `CKSyncEngine` | class | `CKSyncEngine.h` | `CKSyncEngine` (`src/sync_engine.rs:1291`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineDelegate` | protocol | `CKSyncEngine.h` | `CKSyncEngineDelegate` (`src/sync_engine.rs:318`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineFetchChangesContext` | class | `CKSyncEngine.h` | `CKSyncEngineFetchChangesContext` (`src/sync_engine.rs:279`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineFetchChangesOptions` | class | `CKSyncEngine.h` | `CKSyncEngineFetchChangesOptions` (`src/sync_engine.rs:119`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineFetchChangesScope` | class | `CKSyncEngine.h` | `CKSyncEngineFetchChangesScope` (`src/sync_engine.rs:61`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineSendChangesContext` | class | `CKSyncEngine.h` | `CKSyncEngineSendChangesContext` (`src/sync_engine.rs:299`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineSendChangesOptions` | class | `CKSyncEngine.h` | `CKSyncEngineSendChangesOptions` (`src/sync_engine.rs:251`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineSendChangesScope` | class | `CKSyncEngine.h` | `CKSyncEngineSendChangesScope` (`src/sync_engine.rs:159`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineSyncReason` | enum | `CKSyncEngine.h` | `CKSyncEngineSyncReason` (`src/sync_engine.rs:21`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineConfiguration` | class | `CKSyncEngineConfiguration.h` | `CKSyncEngineConfiguration` (`src/sync_engine.rs:339`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineAccountChangeEvent` | class | `CKSyncEngineEvent.h` | `CKSyncEngineAccountChangeEvent` (`src/sync_engine.rs:876`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineAccountChangeType` | enum | `CKSyncEngineEvent.h` | `CKSyncEngineAccountChangeType` (`src/sync_engine.rs:721`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineDidFetchChangesEvent` | class | `CKSyncEngineEvent.h` | `CKSyncEngineDidFetchChangesEvent` (`src/sync_engine.rs:1096`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineDidFetchRecordZoneChangesEvent` | class | `CKSyncEngineEvent.h` | `CKSyncEngineDidFetchRecordZoneChangesEvent` (`src/sync_engine.rs:1071`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineDidSendChangesEvent` | class | `CKSyncEngineEvent.h` | `CKSyncEngineDidSendChangesEvent` (`src/sync_engine.rs:1126`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineEvent` | class | `CKSyncEngineEvent.h` | `CKSyncEngineEvent` (`src/sync_engine.rs:1158`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineEventType` | enum | `CKSyncEngineEvent.h` | `CKSyncEngineEventType` (`src/sync_engine.rs:703`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineFailedRecordSave` | class | `CKSyncEngineEvent.h` | `CKSyncEngineFailedRecordSave` (`src/sync_engine.rs:781`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineFailedZoneSave` | class | `CKSyncEngineEvent.h` | `CKSyncEngineFailedZoneSave` (`src/sync_engine.rs:801`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineFetchedDatabaseChangesEvent` | class | `CKSyncEngineEvent.h` | `CKSyncEngineFetchedDatabaseChangesEvent` (`src/sync_engine.rs:915`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineFetchedRecordDeletion` | class | `CKSyncEngineEvent.h` | `CKSyncEngineFetchedRecordDeletion` (`src/sync_engine.rs:738`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineFetchedRecordZoneChangesEvent` | class | `CKSyncEngineEvent.h` | `CKSyncEngineFetchedRecordZoneChangesEvent` (`src/sync_engine.rs:938`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineFetchedZoneDeletion` | class | `CKSyncEngineEvent.h` | `CKSyncEngineFetchedZoneDeletion` (`src/sync_engine.rs:761`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineSentDatabaseChangesEvent` | class | `CKSyncEngineEvent.h` | `CKSyncEngineSentDatabaseChangesEvent` (`src/sync_engine.rs:961`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineSentRecordZoneChangesEvent` | class | `CKSyncEngineEvent.h` | `CKSyncEngineSentRecordZoneChangesEvent` (`src/sync_engine.rs:1001`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineStateUpdateEvent` | class | `CKSyncEngineEvent.h` | `CKSyncEngineStateUpdateEvent` (`src/sync_engine.rs:861`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineWillFetchChangesEvent` | class | `CKSyncEngineEvent.h` | `CKSyncEngineWillFetchChangesEvent` (`src/sync_engine.rs:1041`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineWillFetchRecordZoneChangesEvent` | class | `CKSyncEngineEvent.h` | `CKSyncEngineWillFetchRecordZoneChangesEvent` (`src/sync_engine.rs:1056`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineWillSendChangesEvent` | class | `CKSyncEngineEvent.h` | `CKSyncEngineWillSendChangesEvent` (`src/sync_engine.rs:1111`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineZoneDeletionReason` | enum | `CKSyncEngineEvent.h` | `CKSyncEngineZoneDeletionReason` (`src/sync_engine.rs:730`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineRecordZoneChangeBatch` | class | `CKSyncEngineRecordZoneChangeBatch.h` | `CKSyncEngineRecordZoneChangeBatch` (`src/sync_engine.rs:649`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEnginePendingDatabaseChange` | class | `CKSyncEngineState.h` | `CKSyncEnginePendingDatabaseChange` (`src/sync_engine.rs:491`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEnginePendingDatabaseChangeType` | enum | `CKSyncEngineState.h` | `CKSyncEnginePendingDatabaseChangeType` (`src/sync_engine.rs:466`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEnginePendingRecordZoneChange` | class | `CKSyncEngineState.h` | `CKSyncEnginePendingRecordZoneChange` (`src/sync_engine.rs:439`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEnginePendingRecordZoneChangeType` | enum | `CKSyncEngineState.h` | `CKSyncEnginePendingRecordZoneChangeType` (`src/sync_engine.rs:414`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEnginePendingZoneDelete` | class | `CKSyncEngineState.h` | `CKSyncEnginePendingZoneDelete` (`src/sync_engine.rs:544`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEnginePendingZoneSave` | class | `CKSyncEngineState.h` | `CKSyncEnginePendingZoneSave` (`src/sync_engine.rs:514`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineState` | class | `CKSyncEngineState.h` | `CKSyncEngineState` (`src/sync_engine.rs:573`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSyncEngineStateSerialization` | class | `CKSyncEngineState.h` | `CKSyncEngineStateSerialization` (`src/sync_engine.rs:46`) | Local simulation in `src/sync_engine.rs`; never contacts CloudKit |
| `CKSystemSharingUIObserver` | class | `CKSystemSharingUIObserver.h` | `CKSystemSharingUIObserver` (`src/share.rs:811`) | Rust model; its handlers are never registered with CloudKit, so they never fire |

## 🔴 GAPS

None.

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| `CKApplicationPermissionStatus` | enum | `CKContainer.h` | Deprecated discoverability-permission status enum. | `API_DEPRECATED("No longer supported. Please see Sharing CloudKit Data with Other iCloud Users.", macos(10.10, 14.0), ios(8.0, 17.0), tvos(9.0, 17.0), watchos(3.0, 10.0))` |
| `CKOwnerDefaultName` | constant | `CKContainer.h` | Deprecated constant; replaced by `CKCurrentUserDefaultName`. | `API_DEPRECATED_WITH_REPLACEMENT("CKCurrentUserDefaultName", macos(10.10, 10.12), ios(8.0, 10.0), tvos(9.0, 10.0), watchos(3.0, 3.0))` |
| `CKDiscoverAllUserIdentitiesOperation` | class | `CKDiscoverAllUserIdentitiesOperation.h` | Deprecated discoverability operation; explicitly skipped. | `API_DEPRECATED("No longer supported. Please see Sharing CloudKit Data with Other iCloud Users.", macos(10.12, 14.0), ios(10.0, 17.0), watchos(3.0, 10.0))` |
| `CKDiscoverUserIdentitiesOperation` | class | `CKDiscoverUserIdentitiesOperation.h` | Deprecated discoverability operation; explicitly skipped. | `API_DEPRECATED("No longer supported. Please see Sharing CloudKit Data with Other iCloud Users.", macos(10.12, 14.0), ios(10.0, 17.0), tvos(10.0, 17.0), watchos(3.0, 10.0))` |
| `CKFetchRecordChangesOperation` | class | `CKFetchRecordChangesOperation.h` | Deprecated by `CKFetchRecordZoneChangesOperation`. | `API_DEPRECATED_WITH_REPLACEMENT("CKFetchRecordZoneChangesOperation", macos(10.10, 10.12), ios(8.0, 10.0), tvos(9.0, 10.0), watchos(3.0, 3.0))` |
| `CKFetchRecordZoneChangesOptions` | class | `CKFetchRecordZoneChangesOperation.h` | Deprecated by `CKFetchRecordZoneChangesConfiguration`. | `API_DEPRECATED_WITH_REPLACEMENT("CKFetchRecordZoneChangesConfiguration", macos(10.12, 10.14), ios(10.0, 12.0), tvos(10.0, 12.0), watchos(3.0, 5.0))` |

## Notes

- The audited public surface is fully represented for non-exempt SDK symbols. The crate provides a comprehensive Rust API for CloudKit operations, including record management, sync engine, sharing, notifications, and all supported operations.
- All EXEMPT symbols are formally deprecated APIs marked with `API_DEPRECATED` or `API_DEPRECATED_WITH_REPLACEMENT` in the 26.2 SDK headers.
- The wrapper combines Rust-safe abstractions (src/*.rs) with Swift runtime integration (swift-bridge/*) to provide zero-cost bindings to CloudKit.framework.
