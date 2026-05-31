use std::io::{self, Write};

fn main() {
    let mut input_buffer = String::new();

    loop {
        print_prompt();

        io::stdout().flush().unwrap();

        input_buffer.clear();

        match io::stdin().read_line(&mut input_buffer) {
            Ok(0) => {
                println!("\nEOF reached. Exiting.");
                break;
            }
            Ok(_) => {
                let command = input_buffer.trim();

                if command == ".exit" {
                    println!("Exiting.");
                    break;
                }
            }
            Err(error) => {
                println!("Error reading input: {}", error);
                break;
            }
        }
    }
}

fn print_prompt() {
    print!("db > ");
}
