use std::path::Path;
use super::memory_page::MemoryPage;
use super::slotted_page::SlottedPage;
use super::buffer_manager::BufferManager;
use super::disk_manager::{DiskManager, PageId};
use super::slotted_page::PAGE_SIZE;

pub struct AccessManager {
    buffer_manager: BufferManager,
    disk_manager: DiskManager,
}

impl AccessManager {
    pub fn new(path: impl AsRef<Path>) -> Option<Self> {
        let buffer_manager = BufferManager::new();
        let mut disk_manager = DiskManager::new(path).ok()?;
        let next_page_id = disk_manager.next_page_id();
        let root_page = match next_page_id {
            PageId(0) => {}
            _ => {
                let mut buf = [0 as u8; PAGE_SIZE];
                let root_page = disk_manager.fetch_page(PageId(0), &mut buf).ok()?;
                MemoryPage::from(buf);
            }
        };

        Some(Self {
            buffer_manager,
            disk_manager,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::AccessManager;

    #[test]
    fn test() {
        let path = "test.idb";
        let manager = AccessManager::new(path);
        assert_eq!(manager.is_some(), true)
    }
}