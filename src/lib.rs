pub mod components;
pub mod consts;
pub mod errors;
pub mod util;

use components::{pager::Pager, row::Row, table::Table};
use errors::{ExecuteError, MetaCommandError, ParseError};
use util::{do_meta_command, execute_statement, prepare_statement};

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
            Err(ExecuteError::DuplicateKey) => Some("Error: Duplicate key.".to_string()),
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
