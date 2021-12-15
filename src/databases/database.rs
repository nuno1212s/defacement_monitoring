
pub trait WebsiteDefacementDB {

    fn insert_tracked_page(&self, page: &str) -> Result<u32, String>;

    fn list_all_tracked_pages(&self) -> Result<Vec<String>, String>;

    fn del_tracked_page(&self, page : &str) -> Result<bool, String>;

    fn read_dom_for_page(&self, page: &str) -> Result<String, String>;

    fn update_dom_for_page(&self, page: &str, page_dom: &str) -> Result<(), String>;

}
