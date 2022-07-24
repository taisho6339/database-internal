use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

pub struct PageId(pub u32);

impl PageId {
    pub fn to_u32(self) -> u32 {
        self.0
    }
}

pub struct DiskManager {
    file: File,
    next_page_id: u32,
}

impl DiskManager {
    fn new(file_path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path)?;
        Ok(DiskManager {
            file,
            next_page_id: 0,
        })
    }
}
