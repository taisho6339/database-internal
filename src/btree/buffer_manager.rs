use std::cell::RefCell;
use std::rc::Rc;

use thiserror::Error;

use crate::btree::access_manager::BufferId;
use crate::btree::slotted_page::SlottedPage;

#[derive(Debug, Error)]
pub enum Error {
    #[error("no free buffer available in buffer pool")]
    NoFreeBuffer,
}

#[derive(Debug)]
pub struct PageBuffer {
    pub is_dirty: bool,
    pub page: RefCell<SlottedPage>,
}

impl Default for PageBuffer {
    fn default() -> Self {
        Self {
            is_dirty: false,
            page: RefCell::new(SlottedPage::new(0)),
        }
    }
}

#[derive(Debug, Default)]
pub struct BufferItem {
    is_pinned: bool,
    usage_count: u32,
    buffer: Rc<RefCell<PageBuffer>>,
}

pub struct BufferManager {
    // Clock-wise algorithm
    cache: Vec<BufferItem>,
    next_check_id: BufferId,
}

impl BufferManager {
    pub fn new(size: usize) -> Self {
        let mut cache = vec![];
        let next_check_id = BufferId(0);
        cache.resize_with(size, Default::default);
        Self {
            cache,
            next_check_id,
        }
    }

    // TODO: implement concurrency control later
    pub fn add_page(&mut self, item: RefCell<SlottedPage>) -> Result<BufferId, Error> {
        let mut pinned_count = 0;
        loop {
            if pinned_count >= self.cache.len() {
                return Err(Error::NoFreeBuffer);
            }
            let item = &mut self.cache[self.next_check_id.to_usize()];
            if item.is_pinned {
                pinned_count += 1;
                self.increment_next_buffer_id();
                continue;
            }
            // TODO: CAS
            if item.usage_count > 0 {
                item.usage_count -= 1;
                self.increment_next_buffer_id();
                continue;
            }
            break;
        }
        let buffer_id = self.next_check_id;
        let page_buffer = PageBuffer {
            is_dirty: false,
            page: item,
        };
        let buffer_item = BufferItem {
            usage_count: 0,
            is_pinned: false,
            buffer: Rc::new(RefCell::new(page_buffer)),
        };
        self.cache[buffer_id.to_usize()] = buffer_item;
        self.increment_next_buffer_id();
        Ok(buffer_id)
    }

    pub fn fetch_page(&mut self, buffer_id: &BufferId) -> Option<Rc<RefCell<PageBuffer>>> {
        let index = buffer_id.to_usize();
        if index >= self.cache.len() {
            return None;
        }
        let item = &mut self.cache[index];
        let is_empty = item.buffer.borrow().page.borrow().empty();
        if is_empty {
            return None;
        }
        item.usage_count += 1;

        Some(item.buffer.clone())
    }

    fn increment_next_buffer_id(&mut self) {
        let next_id = ((self.next_check_id.to_usize() + 1) % self.cache.len()) as u32;
        self.next_check_id = BufferId(next_id);
    }
}

#[cfg(test)]
mod tests {
    use crate::btree::slotted_page::MAGIC_NUMBER_LEAF;

    use super::*;

    #[test]
    fn test_increment_next_buffer_id() {
        let mut manager = BufferManager::new(2);
        assert_eq!(manager.next_check_id.to_usize(), 0);
        manager.increment_next_buffer_id();
        assert_eq!(manager.next_check_id.to_usize(), 1);
        manager.increment_next_buffer_id();
        assert_eq!(manager.next_check_id.to_usize(), 0);
    }

    #[test]
    fn test_add_page() {
        let mut manager = BufferManager::new(2);
        assert_eq!(manager.next_check_id.to_usize(), 0);

        // Add
        let page = RefCell::new(SlottedPage::new(MAGIC_NUMBER_LEAF));
        let result = manager.add_page(page);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().to_usize(), 0);
        assert_eq!(manager.next_check_id.to_usize(), 1);

        // Add over the capacity
        let page = RefCell::new(SlottedPage::new(MAGIC_NUMBER_LEAF));
        let result = manager.add_page(page);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().to_usize(), 1);
        assert_eq!(manager.next_check_id.to_usize(), 0);

        let page = RefCell::new(SlottedPage::new(MAGIC_NUMBER_LEAF));
        let result = manager.add_page(page);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().to_usize(), 0);
        assert_eq!(manager.next_check_id.to_usize(), 1);
    }

    #[test]
    fn test_fetch_page() {
        let buffer_id = BufferId(0);
        let mut manager = BufferManager::new(2);
        let ret = manager.fetch_page(&buffer_id);
        assert_eq!(ret.is_none(), true);

        let ret = manager.add_page(RefCell::new(SlottedPage::new(MAGIC_NUMBER_LEAF)));
        assert_eq!(ret.is_ok(), true);
        assert_eq!(ret.unwrap().to_usize(), 0);

        let ret = manager.fetch_page(&buffer_id);
        assert_eq!(ret.is_some(), true);
        let p = ret.unwrap();
        assert_eq!(p.borrow().page.borrow().empty(), false);
        assert_eq!(p.borrow().page.borrow().header_view().magic_number().read(), MAGIC_NUMBER_LEAF);
    }
}