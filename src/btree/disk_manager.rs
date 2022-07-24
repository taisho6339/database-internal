use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use crate::btree::slotted_page::SlottedPage;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct PageId(pub u32);

impl PageId {
    pub fn to_u32(self) -> u32 {
        self.0
    }
    pub fn to_u64(self) -> u64 {
        self.0 as u64
    }
}

pub struct DiskManager {
    file: File,
    next_page_id: PageId,
}

impl DiskManager {
    pub fn new(file_path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path)?;
        Ok(Self {
            file,
            next_page_id: PageId(0),
        })
    }

    pub fn next_page_id(&self) -> PageId {
        self.next_page_id
    }

    pub fn create_page(&self) -> SlottedPage {
        let page = SlottedPage::new();
    }

    pub fn fetch_page(&mut self, page_id: PageId, buf: &mut [u8]) -> io::Result<()> {
        let offset = page_id.to_u64();
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(buf)
    }
}
