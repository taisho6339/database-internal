use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use thiserror::Error;

use crate::btree::slotted_page::{MAGIC_NUMBER_LEAF, SlottedPage};
use crate::buffer_manager::PageBuffer;
use crate::disk_manager::PageId;

pub struct Node {
    page_buffer: Rc<PageBuffer>,
}

// Deleteの実装

// Searchの実装

// ベンチマーク取る
// メモリやCPUボトルネックの特定方法
// Rustの並行プログラミングを学ぶ
// 並行処理化する

impl Node {
    pub fn new(page_buffer: Rc<PageBuffer>) -> Self {
        Self {
            page_buffer
        }
    }

    // pub fn insert(&self, key: &[u8], value: &[u8]) -> anyhow::Result<(), Error> {
    //     let (index, found) = self.find(key);
    //
    //     Ok(())
    // }

    pub fn is_leaf(&self) -> bool {
        let magic_number = self.page_buffer.page.borrow().header_view().magic_number().read();
        return magic_number == MAGIC_NUMBER_LEAF;
    }

    pub fn find(&self, key: &[u8]) -> (u16, bool) {
        let page_ref = self.page_buffer.page.borrow();
        let header_view = page_ref.header_view();
        let number_of_pointers = header_view.number_of_pointers().read();

        let mut start: i32 = 0;
        let mut end: i32 = number_of_pointers as i32 - 1;
        loop {
            if start > end {
                return (start as u16, false);
            }
            if end < 0 {
                return (0, false);
            }
            let mid = (start + end) / 2;
            let cell_view = page_ref.borrow().cell_view(mid as usize);
            let cell_key_length = cell_view.borrow().key_length().read() as usize;
            let cell_body = cell_view.borrow().body();
            let cell_key = &cell_body[..cell_key_length];
            let order = key.cmp(cell_key);
            match order {
                Ordering::Equal => {
                    return (mid as u16, true);
                }
                Ordering::Less => {
                    end = mid - 1;
                    continue;
                }
                Ordering::Greater => {
                    start = mid + 1;
                    continue;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::BorrowMut;
    use std::io::Write;

    use crate::btree::slotted_page::{cell, MAGIC_NUMBER_LEAF, pointer};

    use super::*;

    #[test]
    fn test_find() {
        let number_of_cells: usize = 5;
        let cell_size: usize = 8;
        let mut page = SlottedPage::new(MAGIC_NUMBER_LEAF);
        for i in 1..=number_of_cells {
            let key = ((i * 2) as u16).to_be_bytes();
            let value = (0xffff as u16).to_be_bytes();
            page.add_cell((i - 1) as usize, &key, &value).unwrap();
        }
        let node = Node::new(Rc::new(PageBuffer {
            page: RefCell::new(page),
            is_dirty: false,
        }));
        assert_eq!(node.find(&(2 as u16).to_be_bytes()), (0, true));
        assert_eq!(node.find(&(3 as u16).to_be_bytes()), (1, false));
        assert_eq!(node.find(&(9 as u16).to_be_bytes()), (4, false));
        assert_eq!(node.find(&(1 as u16).to_be_bytes()), (0, false));
        assert_eq!(node.find(&(11 as u16).to_be_bytes()), (5, false));
    }
}