use crate::databases::TrackedPage;

pub mod checksum_comparator;
pub mod diff_comparator;

/// There are various kinds of comparators,
/// Comparators made for static websites that register if there has been any kind of change
/// If these comparators are applied
/// These comparators must take into account the type of page that it is and then
/// use that information to return if the page has been defaced or not.
///
pub trait Comparator<T> : Send + Sync {

    fn name(&self) -> &str;

    fn compare_between(&self, page: &TrackedPage, dom_1: &T, dom_2: &T) -> CompareResult;

}

//The possible results for a given comparator
pub enum CompareResult {

    //Not defaced means that it is 100% sure that the webpage was not defaced
    NotDefaced,
    //Maybe defaced is for when we can't draw any certain conclusions to either end, like
    //With the checksum result for dynamic webpages. If the checksum is equal then we know 100% the
    //page was not defaced, but if it's not equal (which is to be expected since it's a dynamic page)
    //Then we can't be sure that it was defaced
    MaybeDefaced,
    //This is for when we are 100% sure that the webpage was defaced
    Defaced

}