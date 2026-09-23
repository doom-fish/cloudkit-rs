use cloudkit::prelude::*;

#[test]
fn query_builder_keeps_sort_descriptors() {
    let query = CKQuery::match_all("Todo")
        .with_sort_descriptor(SortDescriptor::new("updatedAt", false))
        .with_sort_descriptor(SortDescriptor::new("title", true));

    assert_eq!(query.record_type(), "Todo");
    assert_eq!(query.sort_descriptors().len(), 2);
    assert_eq!(query.sort_descriptors()[0].key(), "updatedAt");
}

#[test]
fn predicate_values_are_bound_as_arguments_not_parsed_as_syntax() {
    let query = CKQuery::new(
        "Todo",
        QueryPredicate::compare(
            "title",
            ComparisonOperator::Equal,
            "O'Brien \" OR TRUEPREDICATE",
        ),
    );

    let format = query
        .predicate_format()
        .expect("typed predicates build without CloudKit entitlements");

    assert_eq!(format, r#"title == "O'Brien \" OR TRUEPREDICATE""#);
}

#[test]
fn compound_predicates_render_supported_operators() {
    let predicate = QueryPredicate::and([
        QueryPredicate::begins_with("name", "Jo"),
        QueryPredicate::not(QueryPredicate::is_in("status", ["done", "archived"])),
        QueryPredicate::between("priority", 1_i64, 5_i64),
        QueryPredicate::list_contains("tags", "home"),
        QueryPredicate::contains_tokens("blue whale"),
        QueryPredicate::compare("score", ComparisonOperator::GreaterThanOrEqual, 2.5),
    ]);

    let format = CKQuery::new("Task", predicate)
        .predicate_format()
        .expect("compound predicate should build");

    assert_eq!(
        format,
        r#"name BEGINSWITH "Jo" AND (NOT status IN {"done", "archived"}) AND priority BETWEEN {1, 5} AND tags CONTAINS "home" AND SELF CONTAINS "blue whale" AND score >= 2.5"#
    );
}

#[test]
fn distance_predicates_and_location_sort_descriptors_attach_to_queries() {
    let query = CKQuery::new(
        "Place",
        QueryPredicate::within_distance("location", 59.33, 18.07, 10.0),
    )
    .with_sort_descriptor(CKLocationSortDescriptor::new("location", 59.33, 18.07));

    let format = query
        .predicate_format()
        .expect("distance predicate and location sort should build");

    assert!(
        format.starts_with("distanceToLocation:fromLocation:(location, <+59.33"),
        "{format}"
    );
    assert!(format.ends_with("< 10"), "{format}");
    assert!(matches!(
        query.sort_descriptors(),
        [QuerySortDescriptor::Location(descriptor)] if descriptor.key() == "location"
    ));
}

#[test]
fn empty_and_matches_everything() {
    assert_eq!(
        CKQuery::new("Todo", QueryPredicate::and([]))
            .predicate_format()
            .expect("empty conjunction should build"),
        "TRUEPREDICATE"
    );
    assert_eq!(
        CKQuery::match_all("Todo").predicate(),
        &QueryPredicate::True
    );
}

#[test]
fn malformed_raw_formats_return_errors_instead_of_aborting() {
    let cases = [
        QueryPredicate::format("title ==", Vec::new()),
        QueryPredicate::format("a == %@ AND b == %@", vec!["x".into()]),
        QueryPredicate::format("%K == %@", vec![1_i64.into(), "x".into()]),
    ];

    for predicate in cases {
        let error = CKQuery::new("Todo", predicate.clone())
            .predicate_format()
            .expect_err("invalid raw predicate formats must fail");
        assert_eq!(
            error.kind(),
            CloudKitErrorCode::BridgeInvalidArgument,
            "{predicate:?}: {error}"
        );
    }
}

#[test]
fn invalid_queries_return_errors_instead_of_aborting() {
    let empty_type = CKQuery::match_all("")
        .predicate_format()
        .expect_err("CKQuery rejects an empty record type");
    assert_eq!(empty_type.kind(), CloudKitErrorCode::BridgeInvalidArgument);
    assert!(empty_type.message.contains("recordType"), "{empty_type}");

    let asset_argument = CKQuery::new(
        "Photo",
        QueryPredicate::compare(
            "file",
            ComparisonOperator::Equal,
            CKAsset::new("/tmp/a.jpg"),
        ),
    )
    .predicate_format()
    .expect_err("assets are not valid predicate arguments");
    assert_eq!(
        asset_argument.kind(),
        CloudKitErrorCode::BridgeInvalidArgument
    );

    let negative_radius = CKQuery::new(
        "Place",
        QueryPredicate::within_distance("location", 0.0, 0.0, -1.0),
    )
    .predicate_format()
    .expect_err("a negative radius is rejected");
    assert_eq!(
        negative_radius.kind(),
        CloudKitErrorCode::BridgeInvalidArgument
    );
}
