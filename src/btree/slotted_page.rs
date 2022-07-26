use std::mem::size_of;
use byteorder::{BigEndian, ReadBytesExt};
use super::disk_manager::PageId;

// 4KiB
pub const PAGE_SIZE: usize = 1024 * 4;
// 0x32DD is a prefix which represents a page
pub const MAGIC_NUMBER_LEAF: u32 = 0x32DD56AA;
pub const MAGIC_NUMBER_INTERNAL: u32 = 0x32DD77AB;
pub const PAGE_VERSION_V1: u8 = 1;

pub struct SlottedPage {
    data: [u8; PAGE_SIZE],
}

pub struct CellPointer(pub u16);

impl CellPointer {
    pub fn from(input: &[u8]) -> Self {
        let pointer = input.as_ref().read_u16::<BigEndian>().unwrap_or(0);
        CellPointer(pointer)
    }
}

/*
 4KiB per a page
       -------------------------------------------------------------------
 0-4   |                        MagicNumber(4b)                          |
       -------------------------------------------------------------------
 4-8   |                     Number of pointers(4b)                      |
       -------------------------------------------------------------------
 8-12  |                     Next overflow page id (4b)                  |
       -------------------------------------------------------------------
 12-16 |                     Cell offset (4b)                            |
       -------------------------------------------------------------------
 16-20 |                        Check sum (4b)                           |
       -------------------------------------------------------------------
 20-...|  Version (1b)  |                  Pointers                      |
       -------------------------------------------------------------------
       |                           Cells                                 |
       -------------------------------------------------------------------
 */
impl SlottedPage {
    pub fn new(magic_number: u32, page_version: u8) -> Self {
        let mut page = SlottedPage { data: [0; PAGE_SIZE] };
        page.write_magic_number(magic_number);
        page.write_version(page_version);
        page.write_check_sum();
        page
    }

    pub fn from(data: [u8; PAGE_SIZE]) -> Self {
        Self {
            data,
        }
    }

    fn check_sum(&self) -> u32 {
        crc32fast::hash(self.data[0..13].as_ref())
    }

    pub fn bytes(&self) -> &[u8] {
        self.data.as_ref()
    }

    pub fn read_magic_number(&self) -> u32 {
        self.data[0..4].as_ref().read_u32::<BigEndian>().unwrap_or(0)
    }

    pub fn read_num_pointer(&self) -> u32 {
        self.data[4..8].as_ref().read_u32::<BigEndian>().unwrap_or(0)
    }

    pub fn read_next_over_flow_page_id(&self) -> PageId {
        PageId(self.data[8..12].as_ref().read_u32::<BigEndian>().unwrap_or(0))
    }

    pub fn read_cell_offset(&self) -> u32 {
        self.data[12..16].as_ref().read_u32::<BigEndian>().unwrap_or(PAGE_SIZE as u32)
    }

    pub fn read_check_sum(&self) -> u32 {
        self.data[16..20].as_ref().read_u32::<BigEndian>().unwrap_or(0)
    }

    pub fn read_version(&self) -> u8 {
        self.data[20..21].as_ref().read_u8().unwrap_or(0)
    }

    pub fn read_pointers(&self) -> Vec<CellPointer> {
        let num = self.read_num_pointer();
        if num == 0 {
            return Vec::<CellPointer>::new();
        }
        let cell_pointer_size = size_of::<CellPointer>();
        let index = cell_pointer_size * num as usize;
        let pointers_ref = self.data[17..index].as_ref();
        pointers_ref
            .chunks(cell_pointer_size)
            .map(|item| CellPointer::from(item))
            .collect::<Vec<CellPointer>>()
    }

    pub fn read_cells(&self) -> Vec<u8> {
        let offset = PAGE_SIZE - self.read_cell_offset() as usize;
        self.data[offset..PAGE_SIZE].as_ref().to_vec()
    }

    pub fn write_magic_number(&mut self, magic_numer: u32) {
        self.data[0..4].copy_from_slice(&magic_numer.to_be_bytes());
    }

    pub fn write_num_pointer(&mut self, num_pointer: u32) {
        self.data[4..8].copy_from_slice(&num_pointer.to_be_bytes())
    }

    pub fn write_next_over_flow_page_id(&mut self, page_id: PageId) {
        self.data[8..12].copy_from_slice(&page_id.to_u32().to_be_bytes().as_ref())
    }

    pub fn write_cell_offset(&mut self, offset: u32) {
        self.data[12..16].copy_from_slice(offset.to_be_bytes().as_ref())
    }

    pub fn write_check_sum(&mut self) {
        let checksum = self.check_sum();
        self.data[16..20].copy_from_slice(checksum.to_be_bytes().as_ref())
    }

    pub fn write_version(&mut self, version: u8) {
        self.data[20..21].copy_from_slice(&version.to_be_bytes())
    }
}


pub trait Cell {
    fn bytes(&self) -> &[u8];
}

pub struct KeyCell {
    data: Vec<u8>,
}

impl KeyCell {
    pub fn write_key_size(&mut self, size: u32) {
        self.data[0..4].copy_from_slice(size.to_be_bytes().as_ref())
    }

    pub fn write_next_page_id(&mut self, page_id: PageId) {
        self.data[4..8].copy_from_slice(page_id.to_u32().to_be_bytes().as_ref())
    }

    pub fn write_key(&mut self, key: &mut Vec<u8>) {
        self.data.append(key)
    }
}

impl Cell for KeyCell {
    fn bytes(&self) -> &[u8] {
        &self.data.as_slice()
    }
}

pub struct KeyValueCell {
    data: Vec<u8>,
}

impl KeyValueCell {
    pub fn write_key_size(&mut self, size: u32) {
        self.data[0..4].copy_from_slice(size.to_be_bytes().as_ref())
    }
    pub fn write_value_size(&mut self, size: u32) {
        self.data[4..8].copy_from_slice(size.to_be_bytes().as_ref())
    }
    pub fn write_key(&mut self, key: &mut Vec<u8>) {
        self.data.append(key)
    }
    pub fn write_value(&mut self, key: &mut Vec<u8>) {
        self.data.append(key)
    }
}

impl Cell for KeyValueCell {
    fn bytes(&self) -> &[u8] {
        &self.data.as_slice()
    }
}

/*
TODO:
    * CellのInsertどうやって実現する？どこにinsertする？insertできるかの判定は？pointerのソートは？free cellの管理は？
    * Pageを定義してserialize、deserializeできるようにする
    * 実際にファイルに書き込み読み込みできるようにする
    * Pageをつなげて、探索ができるようにする
    * Addできるようにする
    * 削除ができるようにする
    * 更新ができるようにする
 */
