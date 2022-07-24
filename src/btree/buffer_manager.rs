use std::collections::HashMap;
use super::disk_manager::PageId;
use super::memory_page::MemoryPage;

pub struct BufferManager {
    pages: HashMap<PageId, MemoryPage>,
}

impl BufferManager {
    pub fn new() -> Self {
        let pages = HashMap::new();
        Self {
            pages
        }
    }
}