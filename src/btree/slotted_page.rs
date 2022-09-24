use std::borrow::{Borrow, BorrowMut};

use binary_layout::define_layout;
use binary_layout::FieldSliceAccess;
use crate::btree::slotted_page::cell::body;

/*
 4KiB per a page
 -------------------------------------------------------------------
 |                        MagicNumber(4b)                          |
 -------------------------------------------------------------------
 |    Number of pointers(2b)     |           Cell offset (2b)      |
 -------------------------------------------------------------------
 |                     Next overflow page id (4b)                  |
 -------------------------------------------------------------------
 |                        Check sum (4b)                           |
 -------------------------------------------------------------------
 |  Version (1b)  |                  Padding                       |
 -------------------------------------------------------------------
 |                          Pointers                               |
 -------------------------------------------------------------------
 |                           Cells                                 |
 -------------------------------------------------------------------
 */

// 4KiB
pub const PAGE_SIZE: usize = 1024 * 4;
pub const HEADER_SIZE: usize = 20;
pub const PAGE_VERSION_V1: u8 = 1;
pub const POINTER_SIZE: usize = 32;
// 0x32DD is a prefix which represents a page
pub const MAGIC_NUMBER_LEAF: u32 = 0x32DD56AA;
pub const MAGIC_NUMBER_INTERNAL: u32 = 0x32DD77AB;

define_layout!(page_header, BigEndian, {
    magic_number: u32,
    number_of_pointers: u16,
    cell_offset: u16,
    next_overflow_page_id: u32,
    check_sum: u32,
    version: u8,
});

define_layout!(page, BigEndian, {
    header: page_header::NestedView,
    body: [u8; PAGE_SIZE - HEADER_SIZE],
});

define_layout!(pointer, BigEndian, {
    cell_offset: u16,
    cell_length: u16,
});

define_layout!(cell, BigEndian, {
    key_length: u16,
    value_length: u16,
    body: [u8],
});

#[derive(Debug)]
pub struct SlottedPage {
    data: [u8; PAGE_SIZE],
}

impl SlottedPage {
    pub fn new(magic_number: u32) -> Self {
        let mut s = Self {
            data: [0; PAGE_SIZE]
        };
        let sum = s.check_sum();
        s.header_view_mut().magic_number_mut().write(magic_number);
        s.header_view_mut().version_mut().write(PAGE_VERSION_V1);
        s.header_view_mut().check_sum_mut().write(sum);
        let offset = (PAGE_SIZE - HEADER_SIZE) as u16;
        s.header_view_mut().cell_offset_mut().write(offset);

        s
    }

    pub fn wrap(data: [u8; PAGE_SIZE]) -> Self {
        Self {
            data
        }
    }

    pub fn empty(&self) -> bool {
        let m = self.header_view().magic_number().read();
        return m != MAGIC_NUMBER_INTERNAL && m != MAGIC_NUMBER_LEAF;
    }

    pub fn to_bytes(&self) -> &[u8; PAGE_SIZE] {
        page::View::new(&self.data).into_storage()
    }

    // Postgres
    // https://github.com/postgres/postgres/blob/2cd2569c72b8920048e35c31c9be30a6170e1410/src/include/storage/checksum_impl.h#L196
    pub fn check_sum(&mut self) -> u32 {
        let old_csum = self.header_view().check_sum().read();

        // Derived from PostgresSQL implementation
        // https://github.com/postgres/postgres/blob/2cd2569c72b8920048e35c31c9be30a6170e1410/src/include/storage/checksum_impl.h#L196
        self.header_view_mut().check_sum_mut().write(0);
        let csum = crc32fast::hash(&self.data);
        self.header_view_mut().check_sum_mut().write(old_csum);

        csum
    }

    pub fn header_view(&self) -> page_header::View<impl AsRef<[u8]> + '_> {
        let page_view = page::View::new(&self.data[..]);
        page_view.into_header()
    }

    pub fn header_view_mut(&mut self) -> page_header::View<impl AsRef<[u8]> + AsMut<[u8]> + '_> {
        let page_view = page::View::new(&mut self.data[..]);
        page_view.into_header()
    }

    pub fn body_view(&self) -> &[u8] {
        page::body::data(&self.data[..])
    }

    pub fn body_view_mut(&mut self) -> &mut [u8] {
        page::body::data_mut(&mut self.data[..])
    }

    pub fn pointer_view(&self, index: usize) -> pointer::View<impl AsRef<[u8]> + '_> {
        let pointer_size = pointer::SIZE.unwrap();
        let offset = index * pointer_size;
        let pointer_bytes = &self.body_view()[offset..(offset + pointer_size)];

        pointer::View::new(pointer_bytes)
    }

    pub fn pointer_view_mut(&mut self, index: usize) -> pointer::View<impl AsRef<[u8]> + AsMut<[u8]> + '_> {
        let pointer_size = pointer::SIZE.unwrap();
        let offset = index * pointer_size;
        let pointer_bytes = &mut self.body_view_mut()[offset..(offset + pointer_size)];

        pointer::View::new(pointer_bytes)
    }

    pub fn cell_view(&self, index: usize) -> cell::View<impl AsRef<[u8]> + '_> {
        let offset = self.pointer_view(index).cell_offset().read() as usize;
        let length = self.pointer_view(index).cell_length().read() as usize;
        let cell_bytes = &self.body_view()[offset..(offset + length)];

        cell::View::new(cell_bytes)
    }

    pub fn cell_view_mut(&mut self, index: usize) -> cell::View<impl AsRef<[u8]> + AsMut<[u8]> + '_> {
        let offset = self.pointer_view(index).cell_offset().read() as usize;
        let length = self.pointer_view(index).cell_length().read() as usize;
        let cell_bytes = &mut self.body_view_mut()[offset..(offset + length)];

        cell::View::new(cell_bytes)
    }

    fn cell_free_space(&self) -> usize {
        let number_of_pointers = self.header_view().number_of_pointers().read();
        let pointers_length = pointer::SIZE.unwrap() * number_of_pointers as usize;
        let cell_offset = self.header_view().cell_offset().read() as usize;

        cell_offset - pointers_length
    }

    // TODO: return Result type
    // Handling cell space compaction
    pub fn add_cell(&mut self, index: usize, key: &[u8], value: &[u8]) {
        let key_size = key.len();
        let value_size = value.len();
        let cell_size = std::mem::size_of::<u16>() * 2 + key_size + value_size;
        if self.cell_free_space() < cell_size {
            // TODO: return some error
            return;
        }

        // Insert a pointers
        let number_of_pointers = self.header_view().number_of_pointers().read();
        let new_pointers_length = ((number_of_pointers + 1) as usize) * pointer::SIZE.unwrap();
        let next_cell_offset = self.header_view().cell_offset().read() as usize;
        let cell_start = (next_cell_offset - cell_size) as usize;

        let tail_start = index * pointer::SIZE.unwrap();
        let tail_end = (number_of_pointers as usize) * pointer::SIZE.unwrap();
        let new_tail_start = tail_start + pointer::SIZE.unwrap();
        self.body_view_mut()[0..new_pointers_length].copy_within(tail_start..tail_end, new_tail_start);
        self.pointer_view_mut(index).cell_offset_mut().write(cell_start as u16);
        self.pointer_view_mut(index).cell_length_mut().write(cell_size as u16);

        // Add a cell
        self.cell_view_mut(index).key_length_mut().write(key_size as u16);
        self.cell_view_mut(index).value_length_mut().write(value_size as u16);
        let mut cell_buffer: Vec<u8> = vec![0; (key_size + value_size) as usize];
        cell_buffer[0..key_size].copy_from_slice(key);
        cell_buffer[key_size..].copy_from_slice(value);
        self.cell_view_mut(index).body_mut().copy_from_slice(&cell_buffer[..]);

        // Update Headers
        self.header_view_mut().cell_offset_mut().write(cell_start as u16);
        self.header_view_mut().number_of_pointers_mut().write(number_of_pointers + 1);
        let crc = self.check_sum();
        self.header_view_mut().check_sum_mut().write(crc);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use super::*;

    #[test]
    fn test_new() {
        let page = SlottedPage::new(MAGIC_NUMBER_LEAF);
        assert_eq!(page.header_view().magic_number().read(), MAGIC_NUMBER_LEAF);
        assert_eq!(page.header_view().version().read(), PAGE_VERSION_V1);
        assert_eq!(page.header_view().check_sum().read(), 3340501009);
        assert_eq!(page.header_view().next_overflow_page_id().read(), 0);
        assert_eq!(page.header_view().number_of_pointers().read(), 0);
        assert_eq!(page.header_view().cell_offset().read(), 0);
    }

    #[test]
    fn test_get_pointer() {
        let mut page = SlottedPage::new(MAGIC_NUMBER_LEAF);
        page.header_view_mut().number_of_pointers_mut().write(2);

        let mut bytes: [u8; 4] = [0; 4];
        bytes[0..2].copy_from_slice(&mut (0x4 as u16).to_be_bytes());
        bytes[2..4].copy_from_slice(&mut (0x4 as u16).to_be_bytes());
        assert_eq!(page.body_view_mut().write(&bytes).is_ok(), true);

        let pointer = page.pointer_view(0);
        assert_eq!(pointer.cell_offset().read(), 4);
        assert_eq!(pointer.cell_length().read(), 4);

        let pointer = page.pointer_view(1);
        assert_eq!(pointer.cell_offset().read(), 0);
        assert_eq!(pointer.cell_length().read(), 0);
    }

    #[test]
    fn test_add_cell() {
        let cell_size: usize = 8;
        let number_of_cells: usize = 5;
        let mut page = SlottedPage::new(MAGIC_NUMBER_LEAF);
        for i in 0..number_of_cells {
            let key = ((i + 1) as u16).to_be_bytes();
            let value = ((i * 2) as u16).to_be_bytes();
            page.add_cell(i as usize, &key, &value);
        }
        assert_eq!(page.header_view().magic_number().read(), MAGIC_NUMBER_LEAF);
        assert_eq!(page.header_view().number_of_pointers().read(), number_of_cells as u16);
        assert_eq!(page.header_view().cell_offset().read(), (PAGE_SIZE - HEADER_SIZE - number_of_cells * cell_size) as u16);
        for i in 0..number_of_cells {
            assert_eq!(page.pointer_view(i).cell_offset().read(), (PAGE_SIZE - HEADER_SIZE - cell_size * (i + 1)) as u16);
            assert_eq!(page.pointer_view(i).cell_length().read(), 8);
            assert_eq!(page.cell_view(i).key_length().read(), 2);
            assert_eq!(page.cell_view(i).value_length().read(), 2);
            let key = ((i + 1) as u16).to_be_bytes();
            let value = ((i * 2) as u16).to_be_bytes();
            assert_eq!(page.cell_view(i).body()[0..2], key);
            assert_eq!(page.cell_view(i).body()[2..4], value);
        }

        // Insert a cell to the head
        let next_cell_offset = page.header_view().cell_offset().read();

        // Check if the new entry is inserted into the head
        page.add_cell(0, &(0 as u16).to_be_bytes(), &(2048 as u16).to_be_bytes());
        assert_eq!(page.header_view().number_of_pointers().read(), (number_of_cells + 1) as u16);
        assert_eq!(page.header_view().cell_offset().read(), (PAGE_SIZE - HEADER_SIZE - (number_of_cells + 1) * cell_size) as u16);
        assert_eq!(page.pointer_view(0).cell_offset().read(), next_cell_offset - cell_size as u16);
        assert_eq!(page.pointer_view(0).cell_length().read(), 8);
        let key = (0 as u16).to_be_bytes();
        let value = (2048 as u16).to_be_bytes();
        assert_eq!(page.cell_view(0).body()[0..2], key);
        assert_eq!(page.cell_view(0).body()[2..4], value);

        // Check if the previous pointers are slided by one
        for i in 1..=number_of_cells {
            let key = (i as u16).to_be_bytes();
            let value = (((i - 1) * 2) as u16).to_be_bytes();
            assert_eq!(page.cell_view(i).body()[0..2], key);
            assert_eq!(page.cell_view(i).body()[2..4], value);
        }
    }
}