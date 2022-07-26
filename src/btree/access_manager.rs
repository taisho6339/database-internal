use std::io;
use std::path::Path;

use super::buffer_manager::BufferManager;
use super::disk_manager::{DiskManager, PageId};
use super::memory_page::MemoryPage;
use super::slotted_page::{MAGIC_NUMBER_INTERNAL, PAGE_SIZE, PAGE_VERSION_V1};
use super::slotted_page::SlottedPage;

pub struct AccessManager {
    pub buffer_manager: BufferManager,
    //FIXME:
    disk_manager: DiskManager,
}

impl AccessManager {
    pub fn initialize_pages(&mut self) -> io::Result<()> {
        let next_page_id = self.disk_manager.next_page_id();
        let root_page = match next_page_id {
            PageId(0) => {
                let page = SlottedPage::new(MAGIC_NUMBER_INTERNAL, PAGE_VERSION_V1);
                MemoryPage::from(&page)
            }
            _ => {
                let mut buf = [0 as u8; PAGE_SIZE];
                self.disk_manager.fetch_page(PageId(0), &mut buf)?;
                let slotted_page = SlottedPage::from(buf);
                MemoryPage::from(&slotted_page)
            }
        };
        self.buffer_manager.add_page(PageId(0), root_page);
        Ok(())
    }

    pub fn new(path: impl AsRef<Path>) -> Option<Self> {
        let mut buffer_manager = BufferManager::new();
        let mut disk_manager = DiskManager::new(path).ok()?;
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
        assert_eq!(manager.is_some(), true);
        assert_eq!(manager.unwrap().initialize_pages().is_ok(), true);
    }
}