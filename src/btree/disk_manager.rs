use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::btree::slotted_page::{page, PAGE_SIZE, SlottedPage};

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

/*
 * TODO: Maybe I need to implement B-Tree File header including next_page_id, free_page_tables and stuff
 */
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

    pub fn next_page_id(&self) -> &PageId {
        &self.next_page_id
    }

    pub fn write_page(&mut self, page_id: &PageId, page: &SlottedPage) -> io::Result<()> {
        let offset = page_id.to_u64();
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(page.to_bytes())
    }

    pub fn fetch_page(&mut self, page_id: &PageId) -> Option<SlottedPage> {
        let offset = page_id.to_u64();
        let mut buf = [0 as u8; PAGE_SIZE];
        self.file.seek(SeekFrom::Start(offset)).ok()?;
        let ret = self.file.read_exact(&mut buf);

        match ret {
            Ok(_) => Some(SlottedPage::wrap(buf)),
            Err(e) => None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::ptr::write;
    use crate::btree::slotted_page::MAGIC_NUMBER_LEAF;
    use super::*;

    const DB_PATH: &str = "test1.idb";

    struct Cleanup;

    impl Drop for Cleanup {
        fn drop(&mut self) {
            fs::remove_file(DB_PATH).expect("failed to remove db file");
        }
    }

    #[test]
    fn work_as_expected() {
        let cleanup = Cleanup;
        let path = DB_PATH;

        assert_eq!(Path::new(path).exists(), false);
        let ret = DiskManager::new(path);
        assert_eq!(ret.is_ok(), true);
        assert_eq!(Path::new(path).exists(), true);

        let mut manager = ret.unwrap();
        let page_id = PageId(0);
        let fetch_ret = manager.fetch_page(&page_id);
        assert_eq!(fetch_ret.is_some(), false);

        let page = SlottedPage::new(MAGIC_NUMBER_LEAF);
        let write_ret = manager.write_page(&page_id, &page);
        assert_eq!(write_ret.is_ok(), true);

        let fetch_ret = manager.fetch_page(&page_id);
        assert_eq!(fetch_ret.is_some(), true);
        let fetched_page = fetch_ret.unwrap();
        assert_eq!(fetched_page.header_view().check_sum().read(), 3340501009);
    }
}