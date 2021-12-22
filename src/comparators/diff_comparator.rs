use difference::Changeset;

///
/// Compare two doms and returns the percentage of changed items.
pub fn compare_dom_with_diff(dom: &str, dom_2: &str) -> f32 {
    let changes = Changeset::new(dom, dom_2, "\n");

    let dom_size = std::cmp::max(dom.len(), dom_2.len());

    let distance: f32 = changes.distance as f32;

    let distance_percent: f32 = (distance * 100.0) / (dom_size as f32);

    distance_percent
}
