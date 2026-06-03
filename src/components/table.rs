use crate::{PAGE_SIZE, Pager, ROW_SIZE, ROWS_PER_PAGE};
use std::io;
use std::path::Path;

pub struct Table {
    pub num_rows: usize,
    pub pager: Pager,
}

impl Table {
    pub fn db_open<P: AsRef<Path>>(filename: P) -> io::Result<Self> {
        let pager = Pager::open(filename)?;
        let num_rows = (pager.file_length as usize) / ROW_SIZE;

        Ok(Table { num_rows, pager })
    }

    pub fn db_close(&mut self) -> io::Result<()> {
        let num_full_pages = self.num_rows / ROWS_PER_PAGE;

        for i in 0..num_full_pages {
            if self.pager.pages[i].is_none() {
                continue;
            }
            self.pager.flush(i, PAGE_SIZE)?;
        }

        let num_additional_rows = self.num_rows % ROWS_PER_PAGE;
        if num_additional_rows > 0 {
            let page_num = num_full_pages;
            if self.pager.pages[page_num].is_some() {
                self.pager.flush(page_num, num_additional_rows * ROW_SIZE)?;
            }
        }

        Ok(())
    }
}
