use std::fs::File;
use std::io::{BufRead, BufReader};

const MAX_IDENTIFIER_LENGTH: usize = 32;

#[repr(u8)]
#[derive(PartialEq)]
enum Token {
    Identifier(String),
    String(String),
    Number(f32),

    Add,
    Sub,
    Mul,
    Div,

    Comma,
    Colon,
    OpenParen,
    ClosedParen
}

#[repr(u8)]
pub enum ErrorKind {
    UnrecognizedCharacter,
    IntegerOverflow,
    TooLongIdentifier,
    RedundantDot,
    UnclosedString,
}

struct Compiler {
    pub filename: String,
    pub lines: Vec<Vec<Token>>,
    pub code: Vec<u8>
}

impl Compiler {
    pub fn new(filename: String) -> Self {
        Self {
            filename,
            lines: Vec::new(),
            code: Vec::new()
        }
    }

    pub fn parse_file(&mut self, file: File) -> Result<(), ErrorKind> {
        let reader = BufReader::new(file);
        let input_lines = reader.lines().map(|line| line.unwrap());
        let mut current_line: usize = 1;

        let mut error: Option<ErrorKind> = None;

        macro_rules! try_set_error {
            ($kind:ident) => {
                if error.is_none() {
                    error = Some(ErrorKind::$kind);
                }
            };
        }
        
        for input_line in input_lines {
            let mut chars = input_line.chars().peekable();
            let mut line: Vec<Token> = Vec::new();

            macro_rules! print_error {
                ($err_kind:ident, $msg:expr) => {{
                    eprintln!("[{}]: line {}:", self.filename, current_line);
                    eprintln!("  error: {}", $msg);
                    try_set_error!($err_kind);
                }};
            }

            macro_rules! to_digit {
                ($ch:ident) => {
                    $ch.to_digit(10).unwrap()
                };
            }

            macro_rules! num_op {
                ($var:ident, $func:ident, $func_expr:expr) => {
                    if let Some(result) = $var.$func($func_expr) {
                        $var = result;
                    } else {
                        print_error!(IntegerOverflow, "integer overflow");
                        break;
                    }
                };
            }

            macro_rules! is_string_token {
                ($ch:expr) => {
                    ($ch == '"' || $ch == '\'')
                };
            }

            macro_rules! is_identifier_token {
                ($ch:expr) => {
                    ($ch.is_ascii_alphabetic() || $ch == '_')
                };
            }

            // `for ch in chars` takes ownership
            while let Some(ch) = chars.next() {
                if ch.is_ascii_whitespace() {
                    continue
                }

                let identifier: bool = is_identifier_token!(ch);
                let number: bool = ch.is_ascii_digit();
                let string: bool = is_string_token!(ch);
                let line_comment: bool = ch == '#';

                let mut new_token: Option<Token> = None;

                if identifier {
                    let mut ident: String = String::with_capacity(MAX_IDENTIFIER_LENGTH);

                    ident.push(ch);

                    while let Some(ch) = chars.peek() {
                        let ch = *ch;

                        if ident.len() >= MAX_IDENTIFIER_LENGTH {
                            print_error!(TooLongIdentifier, format!(
                                "too long identifier (max length is {} symbols)",
                                MAX_IDENTIFIER_LENGTH
                            ));
                            break;
                        }

                        if !is_identifier_token!(ch) {
                            break;
                        }

                        ident.push(ch);
                        chars.next();
                    }

                    let ident = Token::Identifier(ident);
                    new_token = Some(ident);
                }

                if number {
                    let mut num_a: u32 = to_digit!(ch);
                    let mut num_b: u32 = 0;
                    let mut collecting_a = true;

                    while let Some(ch) = chars.peek() {
                        let ch = *ch;

                        if !ch.is_ascii_digit() {
                            if ch == '.' {
                                if collecting_a {
                                    collecting_a = false;
                                    continue;
                                } else {
                                    print_error!(RedundantDot, "redundant dot near number");
                                    break;
                                }
                            } else {
                                break;
                            }
                        }

                        let digit = to_digit!(ch);
                        assert!(digit < 10);

                        if collecting_a {
                            num_op!(num_a, checked_mul, 10);
                            num_op!(num_a, checked_add, digit);
                        } else {
                            num_op!(num_b, checked_mul, 10);
                            num_op!(num_b, checked_add, digit);
                        }

                        chars.next();
                    }

                    let snum: String = format!("{num_a}.{num_b}");
                    let num: f32 = snum.parse().unwrap();

                    new_token = Some(Token::Number(num));
                }

                if string {
                    let mut string: String = String::new();
                    let mut closed = false;

                    while let Some(ch) = chars.next() {
                        if is_string_token!(ch) {
                            closed = true;
                            break;
                        }

                        string.push(ch);
                    }

                    if closed {
                        let string = Token::String(string);
                        new_token = Some(string);
                    } else {
                        print_error!(UnclosedString, "unclosed string");
                    }
                }

                if line_comment {
                    break;
                }

                if new_token.is_none() {
                    new_token = match ch {
                        '+' => Some(Token::Add),
                        '-' => Some(Token::Sub),
                        '*' => Some(Token::Mul),
                        '/' => Some(Token::Div),

                        ',' => Some(Token::Comma),
                        ':' => Some(Token::Colon),
                        '(' => Some(Token::OpenParen),
                        ')' => Some(Token::ClosedParen),

                        // whitespaces and tabs are skipped in the beginning
                        _ => None
                    };
                }

                if let Some(tok) = new_token {
                    line.push(tok);
                } else {
                    print_error!(UnrecognizedCharacter, format!(
                        "unrecognized character: '{}'",
                        ch
                    ));
                }
            }

            if !line.is_empty() && error.is_none() {
                self.lines.push(line);
            }

            current_line += 1;
        }

        if let Some(error) = error {
            Err(error)
        } else {
            Ok(())
        }
    }
}

pub fn compile_from_file(file: File, filename: String) -> Result<Vec<u8>, ErrorKind> {
    let mut compiler = Compiler::new(filename);
    compiler.parse_file(file)?;
    Ok(compiler.code)
}
