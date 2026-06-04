use std::{
    io::{Read, Seek, SeekFrom},
    process,
    time::Instant,
};

use crate::{
    NodeType, Pager, Statement,
    components::{cursor::Cursor, row::Row, table::Table},
    consts::*,
    errors::{ExecuteError, MetaCommandError, ParseError},
};

pub fn leaf_node_num_cells(node: &[u8; PAGE_SIZE]) -> u32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&node[LEAF_NODE_NUM_CELLS_OFFSET..LEAF_NODE_NUM_CELLS_OFFSET + 4]);
    u32::from_le_bytes(bytes)
}

pub fn set_leaf_node_num_cells(node: &mut [u8; PAGE_SIZE], value: u32) {
    node[LEAF_NODE_NUM_CELLS_OFFSET..LEAF_NODE_NUM_CELLS_OFFSET + 4]
        .copy_from_slice(&value.to_le_bytes());
}

pub fn set_node_is_root(node: &mut [u8; PAGE_SIZE], is_root: bool) {
    node[IS_ROOT_OFFSET] = is_root as u8;
}

pub fn initialize_leaf_node(node: &mut [u8; PAGE_SIZE]) {
    node[NODE_TYPE_OFFSET] = NodeType::Leaf as u8;
    set_node_is_root(node, false);
    set_leaf_node_num_cells(node, 0);
}

pub fn leaf_node_key_mut(node: &mut [u8; PAGE_SIZE], cell_num: usize) -> &mut [u8] {
    let start = LEAF_NODE_HEADER_SIZE + (cell_num * LEAF_NODE_CELL_SIZE);
    &mut node[start..start + LEAF_NODE_KEY_SIZE]
}

pub fn leaf_node_value_mut(node: &mut [u8; PAGE_SIZE], cell_num: usize) -> &mut [u8] {
    let start = LEAF_NODE_HEADER_SIZE + (cell_num * LEAF_NODE_CELL_SIZE) + LEAF_NODE_KEY_SIZE;
    &mut node[start..start + LEAF_NODE_VALUE_SIZE]
}

pub fn leaf_node_insert(cursor: &mut Cursor, key: u32, value: &Row) {
    let page = get_page(&mut cursor.table.pager, cursor.page_num);
    let num_cells = leaf_node_num_cells(page);

    if num_cells >= LEAF_NODE_MAX_CELLS as u32 {
        println!("Need to implement splitting a leaf node.");
        process::exit(1);
    }

    let cell_num = cursor.cell_num;
    let num_cells_idx = num_cells as usize;

    if cell_num < num_cells_idx {
        let src_start = LEAF_NODE_HEADER_SIZE + (cell_num + LEAF_NODE_CELL_SIZE);
        let src_end = LEAF_NODE_HEADER_SIZE + (num_cells_idx * LEAF_NODE_CELL_SIZE);
        let dest_start = src_start + LEAF_NODE_CELL_SIZE;
        page.copy_within(src_start..src_end, dest_start);
    }

    set_leaf_node_num_cells(page, num_cells + 1);

    leaf_node_key_mut(page, cell_num).copy_from_slice(&key.to_le_bytes());
    value.serialize(leaf_node_value_mut(page, cell_num));
}

pub fn print_constants() {
    println!("Constants:");
    println!("ROW_SIZE: {}", ROW_SIZE);
    println!("COMMON_NODE_HEADER_SIZE: {}", COMMON_NODE_HEADER_SIZE);
    println!("LEAF_NODE_HEADER_SIZE: {}", LEAF_NODE_HEADER_SIZE);
    println!("LEAF_NODE_CELL_SIZE: {}", LEAF_NODE_CELL_SIZE);
    println!("LEAF_NODE_SPACE_FOR_CELLS: {}", LEAF_NODE_SPACE_FOR_CELLS);
    println!("LEAF_NODE_MAX_CELLS: {}", LEAF_NODE_MAX_CELLS);
}

pub fn print_leaf_node(node: &[u8; PAGE_SIZE]) {
    let num_cells = leaf_node_num_cells(node);
    println!("leaf (size {})", num_cells);
    for i in 0..num_cells as usize {
        let start = LEAF_NODE_HEADER_SIZE + (i * LEAF_NODE_CELL_SIZE);
        let mut key_bytes = [0u8; 4];
        key_bytes.copy_from_slice(&node[start..start + 4]);
        let key = u32::from_le_bytes(key_bytes);
        println!("  - {} : {}", i, key);
    }
}

pub fn do_meta_command(input: &str, table: &mut Table) -> Result<(), MetaCommandError> {
    match input {
        ".exit" => {
            println!("Exiting.");
            if let Err(e) = table.db_close() {
                println!("Error flushing database cache to disk: {}", e);
                process::exit(1);
            }
            process::exit(0);
        }
        _ => Err(MetaCommandError::Unrecognized),
    }
}

pub fn prepare_statement(input: &str) -> Result<Statement, ParseError> {
    match input {
        s if s == "insert" || s.starts_with("insert ") => {
            let parts: Vec<&str> = input.split_whitespace().collect();

            if parts.len() < 4 {
                return Err(ParseError::SyntaxError);
            }

            let id_str = parts[1];
            let username_str = parts[2];
            let email_str = parts[3];

            if id_str.starts_with('-') {
                return Err(ParseError::NegativeId);
            }

            let id: u32 = match id_str.parse() {
                Ok(val) => val,
                Err(_) => return Err(ParseError::SyntaxError),
            };

            if username_str.len() > USERNAME_SIZE || email_str.len() > EMAIL_SIZE {
                return Err(ParseError::StringTooLong);
            }

            let mut username = [0u8; USERNAME_SIZE];
            username[..parts[2].len()].copy_from_slice(parts[2].as_bytes());

            let mut email = [0u8; EMAIL_SIZE];
            email[..parts[3].len()].copy_from_slice(parts[3].as_bytes());

            Ok(Statement::Insert {
                row_to_insert: Row {
                    id,
                    username,
                    email,
                },
            })
        }
        s if s == "select" || s.starts_with("select ") => Ok(Statement::Select),
        _ => Err(ParseError::UnrecognizedStatement),
    }
}

pub fn execute_statement(statement: Statement, table: &mut Table) -> Result<(), ExecuteError> {
    let start_time = Instant::now();

    match statement {
        Statement::Insert { row_to_insert } => {
            let root_page = get_page(&mut table.pager, table.root_page_num);
            let num_cells = leaf_node_num_cells(root_page);

            if num_cells >= LEAF_NODE_MAX_CELLS as u32 {
                return Err(ExecuteError::TableFull);
            }

            let mut cursor = Cursor::table_end(table);
            leaf_node_insert(&mut cursor, row_to_insert.id, &row_to_insert);
        }
        Statement::Select => {
            let mut cursor = Cursor::table_start(table);
            while !cursor.end_of_table {
                let source_slice = cursor.value();
                let deserialized_row = Row::deserialize(source_slice);
                deserialized_row.print();
                cursor.advance();
            }
        }
    }

    let duration = start_time.elapsed().as_secs_f64() * 1000.0;
    println!("Executed. ({:.3}ms)", duration);
    Ok(())
}

pub fn print_prompt() {
    print!("db > ");
}

pub fn get_page(pager: &mut Pager, page_num: usize) -> &mut [u8; PAGE_SIZE] {
    if page_num >= TABLE_MAX_PAGES {
        println!(
            "Tried to fetch page number out of bounds: {} >= {}",
            page_num, TABLE_MAX_PAGES
        );
        process::exit(1);
    }

    if pager.pages[page_num].is_none() {
        let mut page_data = Box::new([0u8; PAGE_SIZE]);
        let mut num_pages = pager.file_length / (PAGE_SIZE as u64);

        if pager.file_length % (PAGE_SIZE as u64) != 0 {
            num_pages += 1;
        }

        if (page_num as u64) < num_pages {
            pager
                .file
                .seek(SeekFrom::Start((page_num * PAGE_SIZE) as u64))
                .unwrap();
            let mut bytes_read = 0;
            while bytes_read < PAGE_SIZE {
                match pager.file.read(&mut page_data[bytes_read..]) {
                    Ok(0) => break,
                    Ok(n) => bytes_read += n,
                    Err(e) => {
                        println!("Error reading database file: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
        pager.pages[page_num] = Some(page_data);

        if page_num >= pager.num_pages {
            pager.num_pages = page_num + 1;
        }
    }
    pager.pages[page_num].as_mut().unwrap()
}
