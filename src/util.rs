use std::{
    io::{Read, Seek, SeekFrom},
    process,
    time::Instant,
};

use crate::{
    Cursor, EMAIL_SIZE, ExecuteError, MetaCommandError, PAGE_SIZE, Pager, ParseError, Row,
    Statement, TABLE_MAX_PAGES, TABLE_MAX_ROWS, Table, USERNAME_SIZE,
};

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
            if table.num_rows >= TABLE_MAX_ROWS {
                return Err(ExecuteError::TableFull);
            }

            let mut cursor = Cursor::table_end(table);
            let target_slice = cursor.value();
            row_to_insert.serialize(target_slice);

            cursor.table.num_rows += 1;
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
    }
    pager.pages[page_num].as_mut().unwrap()
}
