# cloudkit-rs coverage audit (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 140
VERIFIED: 40
GAPS: 94
EXEMPT: 6
COVERAGE_PCT: 29.85%

Objective-C category extensions were folded into their owning types and not counted as separate symbols. Unavailable APIs were filtered out of the audit (`CKFetchNotificationChangesOperation`, `CKMarkNotificationsReadOperation`, `CKModifyBadgeOperation`).

## Audit highlights

- CKSyncEngine family (state/events/delegate surface)
- share acceptance + requester APIs (`CKAcceptSharesOperation`, `CKShareMetadata`, `CKShareRequestAccessOperation`)
- notification object model (`CKNotification*`)
- generic operation/configuration layer (`CKOperation*`)
- exported CloudKit constants (record/share/error keys)

## 🟢 VERIFIED
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

## 🔴 GAPS
| Symbol | Kind | Header | Notes |
| --- | --- | --- | --- |
| `CKAcceptSharesOperation` | class | `CKAcceptSharesOperation.h` | Share-acceptance/metadata flow is not wrapped. |
| `CKAllowedSharingOptions` | class | `CKAllowedSharingOptions.h` | Newer share-sheet option surface is not wrapped. |
| `CKSharingParticipantAccessOption` | enum | `CKAllowedSharingOptions.h` | Newer share-sheet option surface is not wrapped. |
| `CKSharingParticipantPermissionOption` | enum | `CKAllowedSharingOptions.h` | Newer share-sheet option surface is not wrapped. |
| `CKAccountChangedNotification` | constant | `CKContainer.h` | Container convenience constant is not surfaced. |
| `CKApplicationPermissions` | enum | `CKContainer.h` | Legacy application-permission surface is not wrapped. |
| `CKCurrentUserDefaultName` | constant | `CKContainer.h` | Container convenience constant is not surfaced. |
| `CKDatabaseOperation` | class | `CKDatabaseOperation.h` | Generic operation queue/configuration surface is not wrapped. |
| `CKErrorRetryAfterKey` | constant | `CKError.h` | CloudKit error userInfo constant is not surfaced. |
| `CKErrorUserDidResetEncryptedDataKey` | constant | `CKError.h` | CloudKit error userInfo constant is not surfaced. |
| `CKPartialErrorsByItemIDKey` | constant | `CKError.h` | CloudKit error userInfo constant is not surfaced. |
| `CKRecordChangedErrorAncestorRecordKey` | constant | `CKError.h` | CloudKit error userInfo constant is not surfaced. |
| `CKRecordChangedErrorClientRecordKey` | constant | `CKError.h` | CloudKit error userInfo constant is not surfaced. |
| `CKRecordChangedErrorServerRecordKey` | constant | `CKError.h` | CloudKit error userInfo constant is not surfaced. |
| `CKFetchRecordZonesOperation` | class | `CKFetchRecordZonesOperation.h` | Direct zone CRUD helpers exist, but this operation subclass is absent. |
| `CKFetchShareMetadataOperation` | class | `CKFetchShareMetadataOperation.h` | Share-acceptance/metadata flow is not wrapped. |
| `CKFetchShareParticipantsOperation` | class | `CKFetchShareParticipantsOperation.h` | Container convenience participant lookup exists, but not the operation subclass. |
| `CKFetchSubscriptionsOperation` | class | `CKFetchSubscriptionsOperation.h` | Direct subscription CRUD helpers exist, but this operation subclass is absent. |
| `CKFetchWebAuthTokenOperation` | class | `CKFetchWebAuthTokenOperation.h` | Web-auth token operation is not wrapped. |
| `CKLocationSortDescriptor` | class | `CKLocationSortDescriptor.h` | Only generic `SortDescriptor` is exposed; location-aware sorting is missing. |
| `CKModifyRecordZonesOperation` | class | `CKModifyRecordZonesOperation.h` | Direct zone CRUD helpers exist, but this operation subclass is absent. |
| `CKModifySubscriptionsOperation` | class | `CKModifySubscriptionsOperation.h` | Direct subscription CRUD helpers exist, but this operation subclass is absent. |
| `CKDatabaseNotification` | class | `CKNotification.h` | Notification object model is not wrapped. |
| `CKNotification` | class | `CKNotification.h` | Notification object model is not wrapped. |
| `CKNotificationID` | class | `CKNotification.h` | Notification object model is not wrapped. |
| `CKNotificationType` | enum | `CKNotification.h` | Notification object model is not wrapped. |
| `CKQueryNotification` | class | `CKNotification.h` | Notification object model is not wrapped. |
| `CKQueryNotificationReason` | enum | `CKNotification.h` | Notification object model is not wrapped. |
| `CKRecordZoneNotification` | class | `CKNotification.h` | Notification object model is not wrapped. |
| `CKOperation` | class | `CKOperation.h` | Generic operation queue/configuration surface is not wrapped. |
| `CKOperationConfiguration` | class | `CKOperation.h` | Generic operation queue/configuration surface is not wrapped. |
| `CKOperationGroup` | class | `CKOperationGroup.h` | Generic operation queue/configuration surface is not wrapped. |
| `CKOperationGroupTransferSize` | enum | `CKOperationGroup.h` | Generic operation queue/configuration surface is not wrapped. |
| `CKQueryOperationMaximumResults` | constant | `CKQueryOperation.h` | Maximum-results constant is not surfaced. |
| `CKRecordCreationDateKey` | constant | `CKRecord.h` | Exported record/share constant is not surfaced. |
| `CKRecordCreatorUserRecordIDKey` | constant | `CKRecord.h` | Exported record/share constant is not surfaced. |
| `CKRecordKeyValueSetting` | protocol | `CKRecord.h` | Encrypted-values/key-value protocol is not exposed. |
| `CKRecordLastModifiedUserRecordIDKey` | constant | `CKRecord.h` | Exported record/share constant is not surfaced. |
| `CKRecordModificationDateKey` | constant | `CKRecord.h` | Exported record/share constant is not surfaced. |
| `CKRecordParentKey` | constant | `CKRecord.h` | Exported record/share constant is not surfaced. |
| `CKRecordRecordIDKey` | constant | `CKRecord.h` | Exported record/share constant is not surfaced. |
| `CKRecordShareKey` | constant | `CKRecord.h` | Exported record/share constant is not surfaced. |
| `CKRecordTypeUserRecord` | constant | `CKRecord.h` | Exported record/share constant is not surfaced. |
| `CKRecordZoneDefaultName` | constant | `CKRecordZone.h` | Exported record/share constant is not surfaced. |
| `CKRecordNameZoneWideShare` | constant | `CKShare.h` | Exported record/share constant is not surfaced. |
| `CKRecordTypeShare` | constant | `CKShare.h` | Exported record/share constant is not surfaced. |
| `CKShareThumbnailImageDataKey` | constant | `CKShare.h` | Exported record/share constant is not surfaced. |
| `CKShareTitleKey` | constant | `CKShare.h` | Exported record/share constant is not surfaced. |
| `CKShareTypeKey` | constant | `CKShare.h` | Exported record/share constant is not surfaced. |
| `CKShareAccessRequester` | class | `CKShareAccessRequester.h` | Share requester / system-sharing UI surface is not wrapped. |
| `CKShareBlockedIdentity` | class | `CKShareBlockedIdentity.h` | Share requester / system-sharing UI surface is not wrapped. |
| `CKShareMetadata` | class | `CKShareMetadata.h` | Share-acceptance/metadata flow is not wrapped. |
| `CKShareParticipantType` | enum | `CKShareParticipant.h` | Participant type enum is not wrapped. |
| `CKShareRequestAccessOperation` | class | `CKShareRequestAccessOperation.h` | Share requester / system-sharing UI surface is not wrapped. |
| `CKSyncEngine` | class | `CKSyncEngine.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineDelegate` | protocol | `CKSyncEngine.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineFetchChangesContext` | class | `CKSyncEngine.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineFetchChangesOptions` | class | `CKSyncEngine.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineFetchChangesScope` | class | `CKSyncEngine.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineSendChangesContext` | class | `CKSyncEngine.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineSendChangesOptions` | class | `CKSyncEngine.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineSendChangesScope` | class | `CKSyncEngine.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineSyncReason` | enum | `CKSyncEngine.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineConfiguration` | class | `CKSyncEngineConfiguration.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineAccountChangeEvent` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineAccountChangeType` | enum | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineDidFetchChangesEvent` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineDidFetchRecordZoneChangesEvent` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineDidSendChangesEvent` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineEvent` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineEventType` | enum | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineFailedRecordSave` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineFailedZoneSave` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineFetchedDatabaseChangesEvent` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineFetchedRecordDeletion` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineFetchedRecordZoneChangesEvent` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineFetchedZoneDeletion` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineSentDatabaseChangesEvent` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineSentRecordZoneChangesEvent` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineStateUpdateEvent` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineWillFetchChangesEvent` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineWillFetchRecordZoneChangesEvent` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineWillSendChangesEvent` | class | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineZoneDeletionReason` | enum | `CKSyncEngineEvent.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineRecordZoneChangeBatch` | class | `CKSyncEngineRecordZoneChangeBatch.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEnginePendingDatabaseChange` | class | `CKSyncEngineState.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEnginePendingDatabaseChangeType` | enum | `CKSyncEngineState.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEnginePendingRecordZoneChange` | class | `CKSyncEngineState.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEnginePendingRecordZoneChangeType` | enum | `CKSyncEngineState.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEnginePendingZoneDelete` | class | `CKSyncEngineState.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEnginePendingZoneSave` | class | `CKSyncEngineState.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineState` | class | `CKSyncEngineState.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSyncEngineStateSerialization` | class | `CKSyncEngineState.h` | Entire `CKSyncEngine` family is currently absent. |
| `CKSystemSharingUIObserver` | class | `CKSystemSharingUIObserver.h` | Share requester / system-sharing UI surface is not wrapped. |

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| `CKApplicationPermissionStatus` | enum | `CKContainer.h` | Deprecated discoverability-permission status enum. | `} API_DEPRECATED("No longer supported. Please see Sharing CloudKit Data with Other iCloud Users.", macos(10.10, 14.0), ios(8.0, 17.0), tvos(9.0, 17.0), watchos(3.0, 10.0));` |
| `CKOwnerDefaultName` | constant | `CKContainer.h` | Deprecated constant; replaced by `CKCurrentUserDefaultName`. | `CK_EXTERN NSString * const CKOwnerDefaultName API_DEPRECATED_WITH_REPLACEMENT("CKCurrentUserDefaultName", macos(10.10, 10.12), ios(8.0, 10.0), tvos(9.0, 10.0), watchos(3.0, 3.0));` |
| `CKDiscoverAllUserIdentitiesOperation` | class | `CKDiscoverAllUserIdentitiesOperation.h` | Deprecated discoverability operation; explicitly skipped. | `API_DEPRECATED("No longer supported. Please see Sharing CloudKit Data with Other iCloud Users.", macos(10.12, 14.0), ios(10.0, 17.0), watchos(3.0, 10.0))` |
| `CKDiscoverUserIdentitiesOperation` | class | `CKDiscoverUserIdentitiesOperation.h` | Deprecated discoverability operation; explicitly skipped. | `API_DEPRECATED("No longer supported. Please see Sharing CloudKit Data with Other iCloud Users.", macos(10.12, 14.0), ios(10.0, 17.0), tvos(10.0, 17.0), watchos(3.0, 10.0))` |
| `CKFetchRecordChangesOperation` | class | `CKFetchRecordChangesOperation.h` | Deprecated by `CKFetchRecordZoneChangesOperation`. | `API_DEPRECATED_WITH_REPLACEMENT("CKFetchRecordZoneChangesOperation", macos(10.10, 10.12), ios(8.0, 10.0), tvos(9.0, 10.0), watchos(3.0, 3.0))` |
| `CKFetchRecordZoneChangesOptions` | class | `CKFetchRecordZoneChangesOperation.h` | Deprecated by `CKFetchRecordZoneChangesConfiguration`. | `API_DEPRECATED_WITH_REPLACEMENT("CKFetchRecordZoneChangesConfiguration", macos(10.12, 10.14), ios(10.0, 12.0), tvos(10.0, 12.0), watchos(3.0, 5.0))` |

## Notes

- Direct CRUD/builders cover a useful CloudKit subset, but much of the broader framework surface remains unwrapped.
- The largest uncovered area is the modern `CKSyncEngine` family introduced for higher-level sync orchestration.
- Several database/share operation subclasses remain absent even where the crate offers narrower convenience methods.
