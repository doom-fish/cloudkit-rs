use cloudkit::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = CKQuery::new(
        "Todo",
        QueryPredicate::and([
            QueryPredicate::compare(
                "title",
                ComparisonOperator::Equal,
                "O'Brien \" OR TRUEPREDICATE",
            ),
            QueryPredicate::compare("priority", ComparisonOperator::GreaterThan, 2_i64),
        ]),
    )
    .with_sort_descriptor(SortDescriptor::new("title", true))
    .with_sort_descriptor(CKLocationSortDescriptor::new("location", 59.33, 18.07));
    println!(
        "record_type={} predicate={} sorts={}",
        query.record_type(),
        query.predicate_format()?,
        query.sort_descriptors().len()
    );
    println!("✅ query area OK");
    Ok(())
}
