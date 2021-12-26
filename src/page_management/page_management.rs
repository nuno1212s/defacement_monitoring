use std::sync::{Arc, Mutex};
use crate::databases::{StoredDom, TrackedPage, UserDB, WebsiteDefacementDB};

/*
60 minutes between checks
 */
const TIME_BETWEEN_CHECKS: u128 = 60 * 60 * 1000;

pub struct PageManager<T, V> where
    T: WebsiteDefacementDB + Send + Sync,
    V: UserDB + Send + Sync {
    currently_analysing_pages: Mutex<Vec<TrackedPage>>,
    tracked_page_db: T,
    user_db: V,
}

impl<T, V> PageManager<T, V>
    where T: WebsiteDefacementDB + Send + Sync + 'static,
          V: UserDB + Send + Sync + 'static {
    pub fn new(tracked_page_db: T, user_db: V) -> Self {
        Self {
            currently_analysing_pages: Mutex::new(Vec::new()),
            tracked_page_db,
            user_db,
        }
    }

    pub fn start(self: Arc<Self>) {
        let result = self.tracked_page_db()
            .list_all_pages_not_checked_for(TIME_BETWEEN_CHECKS);

        match result {
            Ok(mut pages_not_checked) => {
                let mut lock_guard = self.currently_analysing_pages.lock().unwrap();

                for page in pages_not_checked {
                    let page_clone = page.clone();

                    let self_res = self.clone();

                    tokio::spawn(async move {
                        let result_doms = self_res.tracked_page_db()
                            .read_doms_for_page(&page_clone);
                    });

                    lock_guard.push(page);
                }
            }
            Err(e) => {
                println!("Failed to check pages because of error: {}", e)
            }
        }
    }

    pub fn verify_page(&self, page: &TrackedPage, stored_dom: &StoredDom) -> bool {

        true

    }

    pub fn tracked_page_db(&self) -> &T {
        &self.tracked_page_db
    }
    pub fn user_db(&self) -> &V {
        &self.user_db
    }
}