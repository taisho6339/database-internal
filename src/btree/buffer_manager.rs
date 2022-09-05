use std::cell::RefCell;
use std::collections::HashMap;

use crate::btree::slotted_page::SlottedPage;

use super::disk_manager::PageId;

// TODO: Implement LRU or 2Q or LFU or Clock cache later
pub struct BufferManager {
    pages: HashMap<PageId, Buffer>,
}

// TODO: HashMapはimmutableなので、offsetの参照だけにして、Vectorで実データを管理するようにする

pub struct Buffer {
    pub is_dirty: bool,
    pub page: RefCell<SlottedPage>,
}

impl BufferManager {
    pub fn new() -> Self {
        let pages = HashMap::new();
        Self {
            pages
        }
    }

    pub fn add_page(&mut self, page_id: PageId, item: RefCell<SlottedPage>) {
        let buffer = Buffer {
            is_dirty: false,
            page: item,
        };
        self.pages.insert(page_id, buffer);
    }

    pub fn fetch_page(&self, page_id: &PageId) -> Option<&Buffer> {
        self.pages.get(page_id)
    }
}