
pub trait WebsiteDefacementDB {

    fn insert_tracked_page(&self, page: &str);

    fn list_all_tracked_pages(&self) -> Vec<String>;

    fn del_tracked_page(&self, page : &str) -> bool;

    fn read_dom_for_page(&self, page: &str) -> String;

    fn update_dom_for_page(&self, page: &str, page_dom: &str);

}
