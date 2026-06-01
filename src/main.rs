use rs_sqlite::{Table, handle_input, print_prompt};
use std::io::{self, Write};

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
                if let Some(output) = handle_input(&input, &mut table) {
                    println!("{}", output);
                }
            }
            Err(error) => {
                println!("Error reading input: {}", error);
                break;
            }
        }
    }
}
