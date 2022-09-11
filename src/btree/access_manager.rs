use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::btree::buffer_manager::{Error, PageBuffer};
use crate::btree::slotted_page::MAGIC_NUMBER_LEAF;

use super::buffer_manager::BufferManager;
use super::disk_manager::{DiskManager, PageId};
use super::slotted_page::SlottedPage;

pub struct AccessManager {
    //FIXME:
    disk_manager: DiskManager,
    buffer_manager: BufferManager,
    buffer_table: HashMap<PageId, BufferId>,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct BufferId(pub u32);

impl BufferId {
    pub fn to_usize(&self) -> usize { self.0 as usize }
    pub fn to_u32(self) -> u32 {
        self.0
    }
    pub fn to_u64(self) -> u64 {
        self.0 as u64
    }
    pub fn increment_id(self) -> BufferId {
        BufferId(self.0 + 1)
    }
}

impl AccessManager {
    pub fn new(path: impl AsRef<Path>) -> Option<Self> {
        let mut disk_manager = DiskManager::new(path).ok()?;
        let mut buffer_manager = BufferManager::new(10);
        let table = HashMap::new();
        Some(Self {
            disk_manager,
            buffer_manager,
            buffer_table: table,
        })
    }

    pub fn initialize(&mut self) -> Result<(), Error> {
        let page_id = PageId(0);
        let root_page = self.disk_manager.fetch_page(&page_id);
        let p = match root_page {
            Some(p) => {
                RefCell::new(p)
            }
            None => {
                RefCell::new(SlottedPage::new(MAGIC_NUMBER_LEAF))
            }
        };
        let buffer_id = self.buffer_manager.add_page(p)?;
        self.buffer_table.insert(page_id, buffer_id);

        Ok(())
    }
    //
    // pub fn fetch_page(&mut self, page_id: &PageId) -> Result<Rc<RefCell<PageBuffer>>, Error> {
    //     if let Some(&buffer_id) = self.buffer_table.get(page_id) {
    //         if let Some(buffer) = self.buffer_manager.fetch_page(&buffer_id) {
    //             return Some(buffer.clone());
    //         }
    //     }
    //     if let Some(page) = self.disk_manager.fetch_page(page_id) {
    //         let ret = self.buffer_manager.add_page(RefCell::new(page));
    //         if ret.is
    //         return Some(Rc);
    //     }
    //     None
    // }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::AccessManager;

    const DB_PATH: &str = "test_access_manager.idb";

    struct Cleanup;

    impl Drop for Cleanup {
        fn drop(&mut self) {
            fs::remove_file(DB_PATH).expect("failed to remove db file");
        }
    }

    #[test]
    fn test() {
        let cleanup = Cleanup;
        let ret = AccessManager::new(DB_PATH);
        assert_eq!(ret.is_some(), true);
        let mut manager = ret.unwrap();
        assert_eq!(manager.initialize().is_ok(), true);
    }
}