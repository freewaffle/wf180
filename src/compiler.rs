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

            let mut new_token: Option<Token> = None;

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
                new_token = Some(ident);
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
                new_token = Some(Token::Number(num));
            }

            if line_comment {
                break;
            }

            if new_token.is_none() {
                new_token = match ch {
                    '+' => Some(Token::Add),

                    '-' => {
                        if chars.peek().is_some_and(|ch| *ch == '>') {
                            chars.next();
                            Some(Token::RightArrow)
                        } else {
                            Some(Token::Sub)
                        }
                    },

                    '*' => Some(Token::Mul),
                    '/' => Some(Token::Div),

                    ',' => Some(Token::Comma),

                    _ => None
                };
            }

            if let Some(tok) = new_token {
                line.push(tok);
            } else {
                error!(format!("invalid character: [{ch}]"));
            }
        }

        if !line.is_empty() && !bad {
            println!(">>> added!");
            lines.push(line);
        }

        current_line += 1;
    }

    Vec::new()
}
