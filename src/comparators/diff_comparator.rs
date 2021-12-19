use difference::Changeset;

///
/// Compare two doms and returns the percentage of changed items.
pub fn compare_dom_with_diff(dom: &str, dom_2: &str) -> f32 {
    let changes = Changeset::new(dom, dom_2, "\n");

    let dom_size = dom.len();

    let distance = changes.distance;

    let distance_percent: f32 = (distance * 100.0) / dom_size;

    distance_percent
}
