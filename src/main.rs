use rs_sqlite::{components::table::Table, handle_input, util::print_prompt};
use std::env;
use std::io::{self, Write};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Must supply a database filename.");
        process::exit(1);
    }
    let filename = &args[1];

    let mut input = String::new();
    let mut table = match Table::db_open(filename) {
        Ok(t) => t,
        Err(e) => {
            println!("Unable to open database file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    loop {
        print_prompt();
        io::stdout().flush().unwrap();
        input.clear();

        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!("\nEOF reached. Exiting.");
                let _ = table.db_close();
                break;
            }
            Ok(_) => {
                if let Some(output) = handle_input(&input, &mut table) {
                    println!("{}", output);
                }
            }
            Err(error) => {
                println!("Error reading input: {}", error);
                let _ = table.db_close();
                break;
            }
        }
    }
}
