#import <Foundation/Foundation.h>
#import <CloudKit/CloudKit.h>

NS_ASSUME_NONNULL_BEGIN

NSPredicate * _Nullable CKXPredicateWithFormat(
    NSString *format,
    NSArray *arguments,
    NSError * _Nullable * _Nullable error
);

CKQuery * _Nullable CKXQueryCreate(
    NSString *recordType,
    NSPredicate *predicate,
    NSError * _Nullable * _Nullable error
);

CKQuerySubscription * _Nullable CKXQuerySubscriptionCreate(
    NSString *recordType,
    NSPredicate *predicate,
    NSString *subscriptionID,
    CKQuerySubscriptionOptions options,
    NSError * _Nullable * _Nullable error
);

CKRecordZoneSubscription * _Nullable CKXRecordZoneSubscriptionCreate(
    CKRecordZoneID *zoneID,
    NSString *subscriptionID,
    NSError * _Nullable * _Nullable error
);

CKDatabaseSubscription * _Nullable CKXDatabaseSubscriptionCreate(
    NSString *subscriptionID,
    NSError * _Nullable * _Nullable error
);

BOOL CKXSubscriptionSetRecordType(
    CKSubscription *subscription,
    NSString * _Nullable recordType,
    NSError * _Nullable * _Nullable error
);

CKRecordZoneID * _Nullable CKXRecordZoneIDCreate(
    NSString *zoneName,
    NSString *ownerName,
    NSError * _Nullable * _Nullable error
);

CKRecordID * _Nullable CKXRecordIDCreate(
    NSString *recordName,
    CKRecordZoneID *zoneID,
    NSError * _Nullable * _Nullable error
);

CKRecord * _Nullable CKXRecordCreate(
    NSString *recordType,
    CKRecordID *recordID,
    NSError * _Nullable * _Nullable error
);

BOOL CKXRecordSetObject(
    CKRecord *record,
    id _Nullable value,
    NSString *key,
    NSError * _Nullable * _Nullable error
);

BOOL CKXRecordSetParent(
    CKRecord *record,
    CKReference * _Nullable parent,
    NSError * _Nullable * _Nullable error
);

BOOL CKXShareAddParticipant(
    CKShare *share,
    CKShareParticipant *participant,
    NSError * _Nullable * _Nullable error
);

BOOL CKXShareRemoveParticipant(
    CKShare *share,
    CKShareParticipant *participant,
    NSError * _Nullable * _Nullable error
);

BOOL CKXShareSetAllowsAccessRequests(
    CKShare *share,
    BOOL allowsAccessRequests,
    NSError * _Nullable * _Nullable error
) API_AVAILABLE(macos(26.0));

NS_ASSUME_NONNULL_END
