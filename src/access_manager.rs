use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use anyhow::{anyhow, Context, Result};
use thiserror::Error;

use crate::access_manager::Error::InitializeError;
use crate::btree::slotted_page::{MAGIC_NUMBER_LEAF, SlottedPage};
use crate::buffer_manager::{BufferError, BufferId, PageBuffer};

use super::buffer_manager::BufferManager;
use super::disk_manager::{DiskManager, PageId};

pub struct AccessManager {
    //FIXME:
    disk_manager: DiskManager,
    buffer_manager: BufferManager,
    buffer_table: HashMap<PageId, BufferId>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("an error occurs in initializing: {0}")]
    InitializeError(String),
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

    pub fn initialize(&mut self) -> Result<()> {
        let page_id = PageId(0);
        let ret = self.disk_manager.fetch_page(page_id);
        let p = match ret {
            Ok(p) => {
                p
            }
            Err(_) => {
                SlottedPage::new(MAGIC_NUMBER_LEAF)
            }
        };
        let buffer_id = self.buffer_manager.add_page(p).context("failed to add a page").map_err(|e| anyhow!(e))?;
        self.buffer_table.insert(page_id, buffer_id);
        Ok(())
    }

    pub fn fetch_page(&mut self, page_id: PageId) -> Result<Rc<PageBuffer>> {
        if let Some(&buffer_id) = self.buffer_table.get(&page_id) {
            if let Some(buffer) = self.buffer_manager.fetch_page(buffer_id) {
                return Ok(buffer);
            }
        }
        let page = self.disk_manager.fetch_page(page_id)
            .with_context(|| format!("failed to find the page with {:?}", page_id))?;
        let buffer_id = self.buffer_manager.add_page(page).context("failed to add the page").map_err(|e| anyhow!(e))?;
        self.buffer_table.insert(page_id, buffer_id);
        let page_buffer = self.buffer_manager.fetch_page(buffer_id).unwrap();

        Ok(page_buffer)
    }
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