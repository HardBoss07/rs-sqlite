use crate::components::table::Table;
use crate::util::{get_page, leaf_node_num_cells, leaf_node_value_mut};

pub struct Cursor<'a> {
    pub table: &'a mut Table,
    pub page_num: usize,
    pub cell_num: usize,
    pub end_of_table: bool,
}

impl<'a> Cursor<'a> {
    pub fn table_start(table: &'a mut Table) -> Self {
        let root_page_num = table.root_page_num;
        let page = get_page(&mut table.pager, root_page_num);
        let num_cells = leaf_node_num_cells(page) as usize;
        let end_of_table = num_cells == 0;

        Cursor {
            table,
            page_num: root_page_num,
            cell_num: 0,
            end_of_table,
        }
    }

    pub fn table_end(table: &'a mut Table) -> Self {
        let root_page_num = table.root_page_num;
        let page = get_page(&mut table.pager, root_page_num);
        let num_cells = leaf_node_num_cells(page) as usize;

        Cursor {
            table,
            page_num: root_page_num,
            cell_num: num_cells,
            end_of_table: true,
        }
    }

    pub fn advance(&mut self) {
        let page = get_page(&mut self.table.pager, self.page_num);
        let num_cells = leaf_node_num_cells(page) as usize;

        self.cell_num += 1;
        if self.cell_num >= num_cells {
            self.end_of_table = true;
        }
    }

    pub fn value(&mut self) -> &mut [u8] {
        let page = get_page(&mut self.table.pager, self.page_num);
        leaf_node_value_mut(page, self.cell_num)
    }
}
