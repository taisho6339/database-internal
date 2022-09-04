use std::io;
use std::io::ErrorKind;
use std::path::Path;
use crate::btree::slotted_page::MAGIC_NUMBER_LEAF;

use super::buffer_manager::BufferManager;
use super::disk_manager::{DiskManager, PageId};
use super::slotted_page::{MAGIC_NUMBER_INTERNAL, PAGE_SIZE, PAGE_VERSION_V1};
use super::slotted_page::SlottedPage;

pub struct AccessManager {
    //FIXME:
    disk_manager: DiskManager,
    buffer_manager: BufferManager,
}

impl AccessManager {
    pub fn initialize(&mut self) -> io::Result<()> {
        // TODO: is it valid to check whether root page is already created or not by using next_page_id?
        let next_page_id = self.disk_manager.next_page_id();
        let root_page = match next_page_id {
            PageId(0) => {
                SlottedPage::new(MAGIC_NUMBER_LEAF)
            }
            _ => {
                self.disk_manager
                    .fetch_page(&PageId(0))
                    .ok_or(std::io::Error::new(ErrorKind::NotFound, ""))?
            }
        };
        self.buffer_manager.add_page(PageId(0), root_page);
        Ok(())
    }

    pub fn new(path: impl AsRef<Path>) -> Option<Self> {
        let mut disk_manager = DiskManager::new(path).ok()?;
        let mut buffer_manager = BufferManager::new();
        Some(Self {
            disk_manager,
            buffer_manager,
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