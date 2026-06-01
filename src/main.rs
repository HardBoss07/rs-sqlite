use std::{
    io::{self, Write},
    process,
    time::Instant,
};

const USERNAME_SIZE: usize = 32; // same as varchar(32)
const EMAIL_SIZE: usize = 255; // same as varchar(255)

const ID_SIZE: usize = 4;
const ID_OFFSET: usize = 0;
const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

const PAGE_SIZE: usize = 4096;
const TABLE_MAX_PAGES: usize = 100;
const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

enum Statement {
    Insert { row_to_insert: Row },
    Select,
}

enum MetaCommandError {
    Unrecognized,
}

enum ParseError {
    UnrecognizedStatement,
    SyntaxError,
    NegativeId,
    StringTooLong,
}

enum ExecuteError {
    TableFull,
}

struct Table {
    num_rows: usize,
    pages: Vec<Option<Box<[u8; PAGE_SIZE]>>>,
}

impl Table {
    fn new() -> Self {
        let mut pages = Vec::with_capacity(TABLE_MAX_PAGES);
        for _ in 0..TABLE_MAX_PAGES {
            pages.push(None);
        }
        Table { num_rows: 0, pages }
    }
}

#[derive(Debug, Clone)]
struct Row {
    id: u32,
    username: [u8; USERNAME_SIZE],
    email: [u8; EMAIL_SIZE],
}

impl Row {
    fn serialize(&self, destination: &mut [u8]) {
        destination[ID_OFFSET..USERNAME_OFFSET].copy_from_slice(&self.id.to_ne_bytes());
        destination[USERNAME_OFFSET..EMAIL_OFFSET].copy_from_slice(&self.username);
        destination[EMAIL_OFFSET..ROW_SIZE].copy_from_slice(&self.email);
    }

    fn deserialize(source: &[u8]) -> Self {
        let mut id_bytes = [0u8; 4];
        id_bytes.copy_from_slice(&source[ID_OFFSET..USERNAME_OFFSET]);
        let id = u32::from_ne_bytes(id_bytes);

        let mut username = [0u8; USERNAME_SIZE];
        username.copy_from_slice(&source[USERNAME_OFFSET..EMAIL_OFFSET]);

        let mut email = [0u8; EMAIL_SIZE];
        email.copy_from_slice(&source[EMAIL_OFFSET..ROW_SIZE]);

        Self {
            id,
            username,
            email,
        }
    }

    fn print(&self) {
        let valid_username = self.username.split(|&b| b == 0).next().unwrap_or(&[]);
        let valid_email = self.email.split(|&b| b == 0).next().unwrap_or(&[]);

        let username_str = std::str::from_utf8(valid_username).unwrap_or("Error parsing username.");
        let email_str = std::str::from_utf8(valid_email).unwrap_or("Error parsing Email.");

        println!("({}, {}, {})", self.id, username_str, email_str);
    }
}

fn main() {
    let mut input = String::new();
    let mut table = Table::new();

    loop {
        print_prompt();

        io::stdout().flush().unwrap();

        input.clear();

        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!("\nEOF reached. Exiting.");
                break;
            }
            Ok(_) => {
                let raw_input = input.trim();
                if raw_input.is_empty() {
                    continue;
                }

                if raw_input.starts_with('.') {
                    match do_meta_command(raw_input) {
                        Ok(()) => continue,
                        Err(MetaCommandError::Unrecognized) => {
                            println!("Unrecognized command: '{}'", raw_input);
                            continue;
                        }
                    }
                }

                match prepare_statement(raw_input) {
                    Ok(statement) => {
                        match execute_statement(statement, &mut table) {
                            Ok(()) => (),
                            Err(ExecuteError::TableFull) => println!("Error: Table full."),
                        };
                    }
                    Err(ParseError::UnrecognizedStatement) => {
                        println!("Unrecognized keyword at start of '{}'.", raw_input);
                    }
                    Err(ParseError::SyntaxError) => {
                        println!("Syntax error. Could not parse statement.");
                    }
                    Err(ParseError::NegativeId) => {
                        println!("ID must be positive.");
                    }
                    Err(ParseError::StringTooLong) => {
                        println!("Unrecognized keyword at start of '{}'.", raw_input);
                    }
                }
            }
            Err(error) => {
                println!("Error reading input: {}", error);
                break;
            }
        }
    }
}

fn do_meta_command(input: &str) -> Result<(), MetaCommandError> {
    match input {
        ".exit" => {
            println!("Exiting.");
            process::exit(0);
        }
        _ => Err(MetaCommandError::Unrecognized),
    }
}

fn prepare_statement(input: &str) -> Result<Statement, ParseError> {
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

fn execute_statement(statement: Statement, table: &mut Table) -> Result<(), ExecuteError> {
    let start_time = Instant::now();

    match statement {
        Statement::Insert { row_to_insert } => {
            if table.num_rows >= TABLE_MAX_ROWS {
                return Err(ExecuteError::TableFull);
            }

            let row_num = table.num_rows;
            let target_slice = get_row_mut_slice(table, row_num);
            row_to_insert.serialize(target_slice);

            table.num_rows += 1;
        }
        Statement::Select => {
            for i in 0..table.num_rows {
                let source_slice = get_row_slice(table, i);
                let deserialized_row = Row::deserialize(source_slice);
                deserialized_row.print();
            }
        }
    }

    let duration = start_time.elapsed().as_secs_f64() * 1000.0;

    println!("Executed. ({:.3}ms)", duration);
    Ok(())
}

fn print_prompt() {
    print!("db > ");
}

fn get_row_mut_slice(table: &mut Table, row_num: usize) -> &mut [u8] {
    let page_num = row_num / ROWS_PER_PAGE;

    if table.pages[page_num].is_none() {
        table.pages[page_num] = Some(Box::new([0u8; PAGE_SIZE]));
    }

    let page = table.pages[page_num].as_mut().unwrap();
    let row_offset = row_num % ROWS_PER_PAGE;
    let byte_offset = row_offset * ROW_SIZE;

    &mut page[byte_offset..byte_offset + ROW_SIZE]
}

fn get_row_slice(table: &Table, row_num: usize) -> &[u8] {
    let page_num = row_num / ROWS_PER_PAGE;
    let page = table.pages[page_num]
        .as_ref()
        .expect("Attempted to read unallocated page slot");

    let row_offset = row_num % ROWS_PER_PAGE;
    let byte_offset = row_offset * ROW_SIZE;

    &page[byte_offset..byte_offset + ROW_SIZE]
}
