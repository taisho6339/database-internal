use std::collections::HashMap;
use crate::btree::slotted_page::SlottedPage;
use super::disk_manager::PageId;

// TODO: Implement LRU or 2Q or LFU or Clock cache later
pub struct BufferManager {
    pages: HashMap<PageId, SlottedPage>,
}

impl BufferManager {
    pub fn new() -> Self {
        let pages = HashMap::new();
        Self {
            pages
        }
    }

    pub fn add_page(&mut self, page_id: PageId, page: SlottedPage) {
        self.pages.insert(page_id, page);
    }

    pub fn fetch_page(&self, page_id: &PageId) -> Option<&SlottedPage> {
        self.pages.get(page_id)
    }
}