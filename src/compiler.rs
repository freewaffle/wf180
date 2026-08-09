use std::fs::File;
use std::io::{BufRead, BufReader};

const MAX_IDENTIFIER_LENGTH: usize = 32;

#[repr(u8)]
enum Token {
    Identifier(String),
    String(String),
    Float(f32),
    Integer(i32),

    Add,
    Sub,
    Mul,
    Div,

    Dot,
    Comma
}

pub fn compile_from_file(file: File, filename: &String) -> Vec<u8> {
    let reader = BufReader::new(file);
    let input_lines = reader.lines().map(|line| line.unwrap());

    let mut lines: Vec<Vec<Token>> = Vec::new();
    let mut current_line: usize = 1;
    
    for input_line in input_lines {
        println!("\n>>> {input_line}");

        let mut chars = input_line.chars().peekable();
        let mut line: Vec<Token> = Vec::new();

        let mut bad = false;

        macro_rules! error {
            ($msg:expr) => {{
                eprintln!("[{filename}]: line {current_line}:");
                eprintln!("  error: {}", $msg);
                bad = true;
            }};
        }

        // `for ch in chars` takes ownership
        while let Some(ch) = chars.next() {
            let identifier = ch.is_ascii_alphabetic() || ch == '_';
            let line_comment = ch == '#';

            if identifier {
                let mut ident: String = String::with_capacity(MAX_IDENTIFIER_LENGTH);

                ident.push(ch);

                while let Some(ch) = chars.peek() {
                    if ident.len() >= MAX_IDENTIFIER_LENGTH {
                        error!(format!("too long identifier (max length is {MAX_IDENTIFIER_LENGTH} symbols)"));
                        break;
                    }

                    if !ch.is_ascii_alphabetic() {
                        break;
                    }

                    ident.push(*ch);
                    chars.next();
                }

                println!("> ident: {ident}");
            }

            if line_comment {
                break;
            }
        }

        if !line.is_empty() && !bad {
            lines.push(line);
        }

        current_line += 1;
    }

    Vec::new()
}
