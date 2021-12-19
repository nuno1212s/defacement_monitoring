pub mod checksum_comparator;
pub mod diff_comparator;

pub trait Comparator {

    fn compare_between(page1: &str, dom_1: &str, page2: &str, dom_2: &str) -> bool;

}

///
/// There are various kinds of comparators,
/// Comparators made for static websites that register if there has been any kind of change
/// If these comparators are applied
pub enum ComparatorTrait {
    StaticComparator,


}