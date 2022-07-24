use std::mem::size_of;
use super::slotted_page::{SlottedPage, MAGIC_NUMBER_LEAF, CellPointer};
use super::disk_manager::{PageId};

pub enum PageType {
    Branch,
    Leaf,
}

pub struct MemoryPage {
    page_type: PageType,
    next_over_flow_page_id: PageId,
    pointers: Vec<CellPointer>,
    cells: Vec<u8>,
}

impl MemoryPage {
    pub fn from(page: &SlottedPage) -> Self {
        let magic_number = page.read_magic_number();
        let page_type = if magic_number == MAGIC_NUMBER_LEAF {
            PageType::Leaf
        } else {
            PageType::Branch
        };
        let next_over_flow_page_id = page.read_next_over_flow_page_id();
        let pointers = page.read_pointers();
        let cells = page.read_cells();
        MemoryPage {
            page_type,
            next_over_flow_page_id,
            pointers,
            cells,
        }
    }
}