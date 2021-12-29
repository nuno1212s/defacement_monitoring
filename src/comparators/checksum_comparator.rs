use crate::comparators::{Comparator, CompareResult};
use crate::comparators::CompareResult::{Defaced, MaybeDefaced, NotDefaced};
use crate::databases::{TrackedPage, TrackedPageType};

/**
This mod focuses on comparing the checksums of the webpages.
This is only effective for static webpages, however since static webpages are no longer the norm
We only use this as a very basic (but very fast) technique to filter out static webpage attacks
(If the checksum is equal then there's no way the page was defaced)
 */

/*
Compare the fully rendered dom
 */
pub fn comp_doms(initial_dom: &str, current_dom: &str) -> bool {
    let mut digestor = sha1::Sha1::new();
    let mut digestor_current = sha1::Sha1::new();

    digestor.update(initial_dom.as_bytes());
    digestor_current.update(current_dom.as_bytes());

    digestor.digest() == digestor_current.digest()
}

pub struct ChecksumComparator {}

impl ChecksumComparator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Comparator<String> for ChecksumComparator {
    fn name(&self) -> &str {
        "Checksum"
    }

    fn compare_between(&self, page: &TrackedPage, dom_1: &String, dom_2: &String) -> CompareResult {
        return match page.tracked_page_type() {
            TrackedPageType::Static => {
                if comp_doms(dom_1, dom_2) {
                    NotDefaced
                } else {
                    Defaced
                }
            }
            TrackedPageType::Dynamic(_) => {
                if comp_doms(dom_1, dom_2) {
                    //If the docs are equal then it can't have been defaced.
                    NotDefaced
                } else {
                    MaybeDefaced
                }
            }
        };

    }
}