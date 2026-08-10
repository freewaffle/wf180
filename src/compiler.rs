use std::fs::File;
use std::io::{BufRead, BufReader};

const MAX_IDENTIFIER_LENGTH: usize = 32;

#[repr(u8)]
enum Token {
    Identifier(String),
    String(String),
    Number(f32),

    Add,
    Sub,
    Mul,
    Div,

    Comma,
    RightArrow
}

pub fn compile_from_file(file: File, filename: &String) -> Vec<u8> {
    let reader = BufReader::new(file);
    let input_lines = reader.lines().map(|line| line.unwrap());
    let mut current_line: usize = 1;

    let mut lines: Vec<Vec<Token>> = Vec::new();
    
    for input_line in input_lines {
        println!("\n>>> [{input_line}]");

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
            let number = ch.is_ascii_digit();
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

                let ident = Token::Identifier(ident);
                line.push(ident);
            }

            if number {
                macro_rules! to_digit {
                    ($ch:ident) => {
                        $ch.to_digit(10).unwrap()
                    };
                }

                let mut num_a: u32 = to_digit!(ch);
                let mut num_b: u32 = 0;
                let mut collecting_a = true;

                while let Some(ch) = chars.peek() {
                    if !ch.is_ascii_digit() {
                        if *ch == '.' {
                            if collecting_a {
                                collecting_a = false;
                                continue;
                            } else {
                                error!("extraneous dot near number");
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    macro_rules! op {
                        ($var:ident, $func:ident, $func_expr:expr) => {
                            if let Some(result) = $var.$func($func_expr) {
                                $var = result;
                            } else {
                                error!("integer overflow");
                                break;
                            }
                        };
                    }

                    let digit = to_digit!(ch);
                    assert!(digit < 10);

                    if collecting_a {
                        op!(num_a, checked_mul, 10);
                        op!(num_a, checked_add, digit);
                    } else {
                        op!(num_b, checked_mul, 10);
                        op!(num_b, checked_add, digit);
                    }

                    chars.next();
                }

                let snum: String = format!("{num_a}.{num_b}");
                let num: f32 = snum.parse().unwrap();

                println!("> number: {num} [{snum}]");
            }

            if line_comment {
                break;
            }

            // match ...
        }

        if !line.is_empty() && !bad {
            println!(">>> added!");
            lines.push(line);
        }

        current_line += 1;
    }

    Vec::new()
}
