pub trait WebsiteDefacementDB {
    fn insert_tracked_page(&self, page: &str, user_id: u32) -> Result<u32, String>;

    fn list_all_tracked_pages(&self) -> Result<Vec<String>, String>;

    fn del_tracked_page(&self, page: &str) -> Result<bool, String>;

    fn read_doms_for_page(&self, page: &str) -> Result<Vec<String>, String>;

    fn insert_dom_for_page(&self, page: &str, page_dom: &str) -> Result<u32, String>;

    fn update_dom_for_page(&self, page: &str, dom_id: u32, page_dom: &str) -> Result<(), String>;

    fn delete_dom_for_page(&self, page: &str, dom_id: u32) -> Result<bool, String>;
}

pub trait UserDB {

    fn create_user(&self, user_name: &str) -> Result<u32, String>;

    fn get_user_id_for(&self, user_name: &str) -> Result<u32, String>;

    fn delete_user(&self, user_id: u32) -> Result<bool, String>;


}