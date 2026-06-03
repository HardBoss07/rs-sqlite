pub mod components;
pub mod consts;
pub mod errors;
pub mod util;

use components::{cursor::*, pager::*, row::*, table::*};
use consts::*;
use errors::*;
use util::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NodeType {
    Internal = 0,
    Leaf = 1,
}

pub enum Statement {
    Insert { row_to_insert: Row },
    Select,
}

pub fn handle_input(input: &str, table: &mut Table) -> Option<String> {
    let raw_input = input.trim();
    if raw_input.is_empty() {
        return None;
    }

    if raw_input.starts_with('.') {
        match do_meta_command(raw_input, table) {
            Ok(()) => return None,
            Err(MetaCommandError::Unrecognized) => {
                return Some(format!("Unrecognized command: '{}'", raw_input));
            }
        }
    }

    match prepare_statement(raw_input) {
        Ok(statement) => match execute_statement(statement, table) {
            Ok(()) => None,
            Err(ExecuteError::TableFull) => Some("Error: Table full.".to_string()),
        },
        Err(ParseError::NegativeId) => Some("ID must be positive.".to_string()),
        Err(ParseError::StringTooLong) => Some("String is too long.".to_string()),
        Err(ParseError::SyntaxError) => {
            Some("Syntax error. Could not parse statement.".to_string())
        }
        Err(ParseError::UnrecognizedStatement) => {
            Some(format!("Unrecognized keyword at start of '{}'.", raw_input))
        }
    }
}

/*
This part is now obsolete since we abstracted away the manual row slice extraction

fn get_row_mut_slice(table: &mut Table, row_num: usize) -> &mut [u8] {
    let page_num = row_num / ROWS_PER_PAGE;
    let page = get_page(&mut table.pager, page_num);
    let row_offset = row_num % ROWS_PER_PAGE;
    let byte_offset = row_offset * ROW_SIZE;

    &mut page[byte_offset..byte_offset + ROW_SIZE]
}

pub fn get_row_slice(table: &mut Table, row_num: usize) -> &[u8] {
    let page_num = row_num / ROWS_PER_PAGE;
    let page = get_page(&mut table.pager, page_num);

    let row_offset = row_num % ROWS_PER_PAGE;
    let byte_offset = row_offset * ROW_SIZE;

    &page[byte_offset..byte_offset + ROW_SIZE]
}
*/
