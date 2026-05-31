use std::{
    io::{self, Write},
    process,
    time::Instant,
};

enum Statement {
    Insert,
    Select,
}

enum MetaCommandError {
    Unrecognized,
}

enum ParseError {
    UnrecognizedStatement,
}

fn main() {
    let mut input = String::new();

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
                        execute_statement(statement);
                    }
                    Err(ParseError::UnrecognizedStatement) => {
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
        s if s == "insert" || s.starts_with("insert ") => Ok(Statement::Insert),
        s if s == "insert" || s.starts_with("select ") => Ok(Statement::Select),
        _ => Err(ParseError::UnrecognizedStatement),
    }
}

fn execute_statement(statement: Statement) {
    let start_time = Instant::now();

    match statement {
        Statement::Insert => println!("This is where we would do an insert."),
        Statement::Select => println!("This is where we would do a select."),
    }

    let duration = start_time.elapsed().as_secs_f64() * 1000.0;

    println!("Executed. ({:.3}ms)", duration);
}

fn print_prompt() {
    print!("db > ");
}
