use std::{
    fs::{File, OpenOptions},
    io::{self, Seek, SeekFrom, Write},
    path::Path,
};

use crate::{PAGE_SIZE, TABLE_MAX_PAGES};

pub struct Pager {
    pub file: File,
    pub file_length: u64,
    pub pages: Vec<Option<Box<[u8; PAGE_SIZE]>>>,
}

impl Pager {
    pub fn open<P: AsRef<Path>>(filename: P) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)?;

        let file_length = file.metadata()?.len();
        let mut pages = Vec::with_capacity(TABLE_MAX_PAGES);

        for _ in 0..TABLE_MAX_PAGES {
            pages.push(None);
        }

        Ok(Pager {
            file,
            file_length,
            pages,
        })
    }

    pub fn flush(&mut self, page_num: usize, size: usize) -> io::Result<()> {
        if self.pages[page_num].is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Tried to flush a null page.",
            ));
        }

        let page = self.pages[page_num].as_ref().unwrap();
        self.file
            .seek(SeekFrom::Start((page_num * PAGE_SIZE) as u64))?;
        self.file.write_all(&page[0..size])?;
        self.file.flush()?;
        Ok(())
    }
}
