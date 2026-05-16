use cloudkit::prelude::*;

#[test]
fn query_builder_keeps_sort_descriptors() {
    let query = CKQuery::match_all("Todo")
        .with_sort_descriptor(SortDescriptor::new("updatedAt", false))
        .with_sort_descriptor(SortDescriptor::new("title", true));

    assert_eq!(query.record_type(), "Todo");
    assert_eq!(query.sort_descriptors().len(), 2);
}
