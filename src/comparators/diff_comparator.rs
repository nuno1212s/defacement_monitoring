use std::time::Duration;
use difference::Changeset;
use log::{debug, error, trace, warn};
use tokio::time;
use crate::comparators::{Comparator, CompareResult};
use crate::comparators::CompareResult::{Defaced, MaybeDefaced, NotDefaced};
use crate::databases::{TrackedPage, TrackedPageType};
use crate::parsers::Parser;

const ANALYSE_TIME_INTERVAL: Duration = Duration::from_millis(1 * 1000);
//2 times per minute (every 30 secs) for 5 minutes
const DYNAMIC_CHECK_COUNT: u32 = 2 * 5;

pub async fn analyse_dynamic_page<T>(parser: &T, page: &TrackedPage) -> Result<f64, String>
    where T: Parser<String> {
    let mut time_period = time::interval(ANALYSE_TIME_INTERVAL);

    let mut checks = 0;

    let mut received_doms = Vec::with_capacity(DYNAMIC_CHECK_COUNT as usize);

    while checks < DYNAMIC_CHECK_COUNT {
        let dom_res = parser.parse_page(page);

        match dom_res {
            Ok(dom) => {
                received_doms.push(dom);
            }
            Err(e) => {
                error!("Failed to read dom for page {} with ID {}. {}", page.page_url(), page.page_id(), e);
            }
        }

        time_period.tick().await;
        checks += 1;
    }

    let mut diff_threshold_avg: f64 = 0.0;
    let mut comparisons: u32 = 0;

    for dom_ind in 0..received_doms.len() {
        for dom_ind_2 in 0..received_doms.len() {
            if dom_ind == dom_ind_2 { continue; }

            let diff = compare_dom_with_diff(received_doms[dom_ind].as_str(),
                                             received_doms[dom_ind_2].as_str()) as f64;

            diff_threshold_avg += diff;
            comparisons += 1;
        }
    }

    diff_threshold_avg /= comparisons as f64;
    //Give some leeway to the defacement calculator as to not register too many false positives
    diff_threshold_avg *= 1.3;

    warn!("After calculating the difference between 10 samples of the website,\
         the difference threshold for the page {} with ID {} is {}",
             page.page_url(), page.page_id(), diff_threshold_avg);

    return Ok(diff_threshold_avg);
}


///
/// Compare two doms and returns the percentage of changed items.
pub fn compare_dom_with_diff(dom: &str, dom_2: &str) -> f64 {
    let changes = Changeset::new(dom, dom_2, "\n");

    let dom_size = std::cmp::max(dom.len(), dom_2.len());

    let distance: f64 = changes.distance as f64;

    let distance_percent: f64 = (distance * 100.0) / (dom_size as f64);


    distance_percent
}

pub struct DiffComparator {}

impl DiffComparator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Comparator<String> for DiffComparator {
    fn name(&self) -> &str {
        "Diff"
    }

    fn compare_between(&self, page: &TrackedPage, dom_1: &String, dom_2: &String) -> CompareResult {
        return match page.tracked_page_type() {
            TrackedPageType::Static => {
                if compare_dom_with_diff(dom_1, dom_2) > 0.0 {
                    Defaced
                } else {
                    NotDefaced
                }
            }
            TrackedPageType::Dynamic(threshold) => {
                let diff = compare_dom_with_diff(dom_1, dom_2);

                trace!("Distance calculated is {}, needs to be below {}", diff, *threshold);

                if diff > *threshold {
                    Defaced
                } else {
                    MaybeDefaced
                }
            }
        };
    }
}