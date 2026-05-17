use cloudkit::prelude::*;

fn main() {
    let query = CKQuery::match_all("Todo").with_sort_descriptor(SortDescriptor::new("title", true));
    println!(
        "record_type={} predicate={} sorts={}",
        query.record_type(),
        query.predicate_format(),
        query.sort_descriptors().len()
    );
    println!("✅ query area OK");
}
