use difference::Changeset;
use crate::comparators::{Comparator, CompareResult};
use crate::comparators::CompareResult::{Defaced, MaybeDefaced, NotDefaced};
use crate::databases::{TrackedPage, TrackedPageType};

///
/// Compare two doms and returns the percentage of changed items.
pub fn compare_dom_with_diff(dom: &str, dom_2: &str) -> f32 {
    let changes = Changeset::new(dom, dom_2, "\n");

    let dom_size = std::cmp::max(dom.len(), dom_2.len());

    let distance: f32 = changes.distance as f32;

    let distance_percent: f32 = (distance * 100.0) / (dom_size as f32);

    distance_percent
}

pub struct DiffComparator {}

impl DiffComparator {
    pub fn new() -> Self {
        Self{}
    }
}

impl Comparator for DiffComparator {
    fn name(&self) -> &str {
        "Diff"
    }

    fn compare_between(&self, page: &TrackedPage, dom_1: &str, dom_2: &str) -> CompareResult {

        return match page.tracked_page_type() {
            TrackedPageType::Static => {
                if compare_dom_with_diff(dom_1, dom_2) > 0.0 {
                    Defaced
                } else {
                    NotDefaced
                }
            }
            TrackedPageType::Dynamic(threshold) => {
                if compare_dom_with_diff(dom_1, dom_2) > *threshold {
                    Defaced
                } else {
                    MaybeDefaced
                }
            }
        };
    }
}