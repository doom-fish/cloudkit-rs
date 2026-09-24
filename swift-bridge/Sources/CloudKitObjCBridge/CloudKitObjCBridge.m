#import "CloudKitObjCBridge.h"

static void CKXWriteException(NSException *exception, NSError **error) {
    if (error == NULL) {
        return;
    }
    NSString *reason = exception.reason ?: @"";
    NSString *message = [NSString stringWithFormat:@"%@: %@", exception.name, reason];
    *error = [NSError errorWithDomain:@"CloudKitBridge"
                                 code:-1
                             userInfo:@{NSLocalizedDescriptionKey: message}];
}

NSPredicate *CKXPredicateWithFormat(NSString *format, NSArray *arguments, NSError **error) {
    @try {
        return [NSPredicate predicateWithFormat:format argumentArray:arguments];
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return nil;
    }
}

CKQuery *CKXQueryCreate(NSString *recordType, NSPredicate *predicate, NSError **error) {
    @try {
        return [[CKQuery alloc] initWithRecordType:recordType predicate:predicate];
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return nil;
    }
}

CKQuerySubscription *CKXQuerySubscriptionCreate(
    NSString *recordType,
    NSPredicate *predicate,
    NSString *subscriptionID,
    CKQuerySubscriptionOptions options,
    NSError **error
) {
    @try {
        return [[CKQuerySubscription alloc] initWithRecordType:recordType
                                                     predicate:predicate
                                                subscriptionID:subscriptionID
                                                       options:options];
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return nil;
    }
}

CKRecordZoneSubscription *CKXRecordZoneSubscriptionCreate(
    CKRecordZoneID *zoneID,
    NSString *subscriptionID,
    NSError **error
) {
    @try {
        return [[CKRecordZoneSubscription alloc] initWithZoneID:zoneID subscriptionID:subscriptionID];
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return nil;
    }
}

CKDatabaseSubscription *CKXDatabaseSubscriptionCreate(NSString *subscriptionID, NSError **error) {
    @try {
        return [[CKDatabaseSubscription alloc] initWithSubscriptionID:subscriptionID];
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return nil;
    }
}

BOOL CKXSubscriptionSetRecordType(CKSubscription *subscription, NSString *recordType, NSError **error) {
    @try {
        if ([subscription isKindOfClass:[CKRecordZoneSubscription class]]) {
            ((CKRecordZoneSubscription *)subscription).recordType = recordType;
        } else if ([subscription isKindOfClass:[CKDatabaseSubscription class]]) {
            ((CKDatabaseSubscription *)subscription).recordType = recordType;
        }
        return YES;
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return NO;
    }
}

CKRecordZoneID *CKXRecordZoneIDCreate(NSString *zoneName, NSString *ownerName, NSError **error) {
    @try {
        return [[CKRecordZoneID alloc] initWithZoneName:zoneName ownerName:ownerName];
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return nil;
    }
}

CKRecordID *CKXRecordIDCreate(NSString *recordName, CKRecordZoneID *zoneID, NSError **error) {
    @try {
        return [[CKRecordID alloc] initWithRecordName:recordName zoneID:zoneID];
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return nil;
    }
}

CKRecord *CKXRecordCreate(NSString *recordType, CKRecordID *recordID, NSError **error) {
    @try {
        return [[CKRecord alloc] initWithRecordType:recordType recordID:recordID];
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return nil;
    }
}

BOOL CKXRecordSetObject(CKRecord *record, id value, NSString *key, NSError **error) {
    @try {
        [record setObject:value forKey:key];
        return YES;
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return NO;
    }
}

BOOL CKXRecordSetParent(CKRecord *record, CKReference *parent, NSError **error) {
    @try {
        record.parent = parent;
        return YES;
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return NO;
    }
}

BOOL CKXShareAddParticipant(CKShare *share, CKShareParticipant *participant, NSError **error) {
    @try {
        [share addParticipant:participant];
        return YES;
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return NO;
    }
}

BOOL CKXShareRemoveParticipant(CKShare *share, CKShareParticipant *participant, NSError **error) {
    @try {
        [share removeParticipant:participant];
        return YES;
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return NO;
    }
}

BOOL CKXShareSetAllowsAccessRequests(CKShare *share, BOOL allowsAccessRequests, NSError **error) {
    @try {
        share.allowsAccessRequests = allowsAccessRequests;
        return YES;
    } @catch (NSException *exception) {
        CKXWriteException(exception, error);
        return NO;
    }
}
