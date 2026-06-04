use crate::components::pager::Pager;
use crate::consts::PAGE_SIZE;
use crate::util::{get_page, initialize_leaf_node, set_node_is_root};
use std::io;
use std::path::Path;

pub struct Table {
    pub root_page_num: usize,
    pub pager: Pager,
}

impl Table {
    pub fn db_open<P: AsRef<Path>>(filename: P) -> io::Result<Self> {
        let pager = Pager::open(filename)?;

        let mut table = Table {
            root_page_num: 0,
            pager,
        };

        if table.pager.num_pages == 0 {
            let root_node = get_page(&mut table.pager, 0);
            initialize_leaf_node(root_node);
            set_node_is_root(root_node, true);
        }

        Ok(table)
    }

    pub fn db_close(&mut self) -> io::Result<()> {
        for i in 0..self.pager.num_pages {
            if self.pager.pages[i].is_none() {
                continue;
            }
            self.pager.flush(i, PAGE_SIZE)?;
        }
        Ok(())
    }
}
