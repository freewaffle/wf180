use std::fs::File;
use std::io::{BufRead, BufReader};

const MAX_IDENTIFIER_LENGTH: usize = 32;

/* const BUILTIN_TYPES: [&str; 2] = [
    "void",
    "number"
]; */

#[derive(PartialEq, Debug)]
#[repr(u8)]
enum TokenKind {
    Identifier(String),
    DoubleIdentifier(String, String),
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

struct Token {
    pub line_pos: usize,
    pub char_pos: usize,
    pub kind: TokenKind,
    pub line_str: String
}

struct TypedIdentifier {
    pub ident: String,
    pub ty: String
}

#[repr(u8)]
enum CommandKind {
    FunctionHeader {
        name: String,
        args: Vec<TypedIdentifier>,
        return_type: String
    },
    FunctionCall {
        name: String,
        args: Vec<Vec<Token>>
    },
    Return {
        expr: Vec<Token>
    },
    End,
    Let {
        var: TypedIdentifier,
        expr: Vec<Token>
    },
}

struct Command {
    pub kind: CommandKind
}

#[repr(u8)]
pub enum ErrorKind {
    UnrecognizedCharacter,
    IntegerOverflow,
    TooLongIdentifier,
    RedundantDot,
    UnclosedString,
    ExpectedToken,
    RedundantTokens,
}

struct Parser {
    pub filename: String
}

impl Parser {
    #[inline]
    pub fn new(filename: String) -> Self {
        Self {
            filename
        }
    }

    #[must_use]
    pub fn parse_line(&self, line_str: String, line_pos: usize) -> Result<Vec<Token>, ErrorKind> {
        let mut error: Option<ErrorKind> = None;

        macro_rules! try_set_error {
            ($kind:ident) => {
                if error.is_none() {
                    error = Some(ErrorKind::$kind);
                }
            };
        }

        let mut chars = line_str.chars().peekable();
        let mut line: Vec<Token> = Vec::new();
        let mut char_pos: usize = 0;

        macro_rules! print_error {
            ($err_kind:ident, $msg:expr) => {{
                eprintln!("[{}]: line {}:", self.filename, line_pos);
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

        macro_rules! next_char {
            () => {{
                char_pos += 1;
                chars.next()
            }};
        }

        macro_rules! collect_identifier {
            ($start_char:expr) => {{
                let mut ident: String = String::with_capacity(MAX_IDENTIFIER_LENGTH);

                ident.push($start_char);

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
                    next_char!();
                }

                ident
            }};
        }

        macro_rules! get_line_str {
            () => {
                line_str.to_owned()
            };
        }

        macro_rules! token {
            ($kind:ident) => {
                Token {
                    line_pos,
                    char_pos,
                    kind: TokenKind::$kind,
                    line_str: get_line_str!()
                }
            };

            ($kind:ident, $expr:expr) => {
                Token {
                    line_pos,
                    char_pos,
                    kind: TokenKind::$kind($expr),
                    line_str: get_line_str!()
                }
            };
        }

        // `for ch in chars` takes ownership
        while let Some(ch) = next_char!() {
            if ch.is_ascii_whitespace() {
                continue
            }

            let identifier: bool = is_identifier_token!(ch);
            let number: bool = ch.is_ascii_digit();
            let string: bool = is_string_token!(ch);
            let line_comment: bool = ch == '#';

            let mut new_token: Option<Token> = None;

            if identifier {
                let ident: String = collect_identifier!(ch);
                let token = token!(Identifier, ident);
                new_token = Some(token);
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

                    next_char!();
                }

                let snum: String = format!("{num_a}.{num_b}");
                let num: f32 = snum.parse().unwrap();

                let token = token!(Number, num);
                new_token = Some(token);
            }

            if string {
                let mut string: String = String::new();
                let mut closed = false;

                while let Some(ch) = next_char!() {
                    if is_string_token!(ch) {
                        closed = true;
                        break;
                    }

                    string.push(ch);
                }

                if closed {
                    let token = token!(String, string);
                    new_token = Some(token);
                } else {
                    print_error!(UnclosedString, "unclosed string");
                }
            }

            if line_comment {
                break;
            }

            if new_token.is_none() {
                let kind = match ch {
                    '+' => Some(TokenKind::Add),
                    '-' => Some(TokenKind::Sub),
                    '*' => Some(TokenKind::Mul),
                    '/' => Some(TokenKind::Div),

                    ',' => Some(TokenKind::Comma),

                    ':' => {
                        let mut remove_last_token = false;

                        let tok = if chars.peek().is_some_and(|ch| is_identifier_token!(*ch)) {
                            if let Some(token) = line.last()
                            && let TokenKind::Identifier(left_ident) = &token.kind {
                                remove_last_token = true;
                                let ch = next_char!().unwrap();
                                let right_ident = collect_identifier!(ch);
                                TokenKind::DoubleIdentifier(left_ident.to_owned(), right_ident)
                            } else {
                                TokenKind::Colon
                            }
                        } else {
                            TokenKind::Colon
                        };

                        if remove_last_token {
                            line.pop();
                        }

                        Some(tok)
                    },

                    '(' => Some(TokenKind::OpenParen),
                    ')' => Some(TokenKind::ClosedParen),

                    // whitespaces and tabs are skipped in the beginning
                    _ => None
                };

                if let Some(kind) = kind {
                    new_token = Some(Token {
                        line_pos,
                        char_pos,
                        kind,
                        line_str: get_line_str!()
                    });
                } // else None
            }

            if let Some(tok) = new_token {
                if error.is_none() {
                    line.push(tok);
                }
            } else {
                print_error!(UnrecognizedCharacter, format!(
                    "unrecognized character: '{}'",
                    ch
                ));
            }

            char_pos += 1;
        }

        if let Some(error) = error {
            Err(error)
        } else {
            Ok(line)
        }
    }

    #[must_use]
    pub fn parse_file(&self, file: File) -> Result<Vec<Vec<Token>>, ErrorKind> {
        let reader = BufReader::new(file);
        let input_lines = reader.lines().map(|line| line.unwrap());
        let mut current_line: usize = 1;

        let mut lines: Vec<Vec<Token>> = Vec::new();

        let mut error: Option<ErrorKind> = None;
        
        for input_line in input_lines {
            match self.parse_line(input_line, current_line) {
                Ok(line) => {
                    if error.is_none() {
                        lines.push(line);
                    }
                }
                Err(err) => {
                    if error.is_none() {
                        error = Some(err);
                    }
                }
            }

            current_line += 1;
        }

        if let Some(error) = error {
            Err(error)
        } else {
            Ok(lines)
        }
    }

    #[must_use]
    pub fn parse_tokens(&self, tokens: Vec<Vec<Token>>) -> Result<Vec<Command>, ErrorKind> {
        let mut commands: Vec<Command> = Vec::new();
        let mut error: Option<ErrorKind> = None;

        macro_rules! try_set_error {
            ($kind:ident) => {
                if error.is_none() {
                    error = Some(ErrorKind::$kind);
                }
            };
        }
        
        'tokens: for tokens in tokens {
            if tokens.is_empty() {
                continue 'tokens;
            }

            let first_token = tokens.first().unwrap();
            let line_pos = first_token.line_pos;

            let mut new_command: Option<Command> = None;

            macro_rules! print_error {
                /* ($err_kind:ident, $msg:expr, $tokn:expr) => {{
                    let token = &tokens[$tokn];
                    eprintln!("[{}]: line {}:", self.filename, line_pos);
                    eprintln!("  {line_str}");
                    eprint!("  ");
                    for _ in 0..(token.char_pos - 2) {
                        eprint!(" ");
                    }
                    eprintln!("^");
                    eprintln!("  error: {}", $msg);
                    try_set_error!($err_kind);
                    continue 'tokens;
                }}; */

                ($err_kind:ident, $msg:expr) => {{
                    eprintln!("[{}]: line {}:", self.filename, line_pos);
                    eprintln!("  error: {}", $msg);
                    try_set_error!($err_kind);
                    continue 'tokens;
                }};
            }

            macro_rules! is_token_of_type {
                ($pos:expr, $kind:ident) => {
                    (tokens.get($pos).is_some_and(|tok| tok.kind == TokenKind::$kind))
                };
                ($pos:expr, $kind:ident, $expr:expr) => {
                    (tokens.get($pos).is_some_and(|tok| tok.kind == TokenKind::$kind($expr)))
                };
            }

            macro_rules! get_token_value {
                ($index:expr, $kind:ident) => {
                    match tokens.get($index) {
                        Some(Token {
                            kind: TokenKind::$kind(ident),
                            ..
                        }) => Some(ident),
                        _ => None,
                    }
                };
            }

            macro_rules! redundant_tokens_error {
                () => {
                    print_error!(RedundantTokens, "redundant tokens");
                };
            }

            macro_rules! check_redundant_tokens {
                ($start:expr) => {{
                    let len = tokens.len().checked_sub(1).unwrap_or(0);
                    let diff = len.checked_sub($start);
                    if diff.is_some_and(|diff| diff > 0) {
                        redundant_tokens_error!();
                    }
                }};
            }

            if let TokenKind::Identifier(ident) = &first_token.kind {
                // as_str() shouldn't clone string, looks like it just
                // does nothing but changes the type.

                // don't be afraid of `clone`: the `compile_from_file` function,
                // which is preferred compilation way, transfers ownership of
                // produced tokens to this function, dropping them after this
                // function finishes.

                match ident.as_str() {
                    "func" => {
                        let name = if let Some(ident) = get_token_value!(1, Identifier) {
                            ident.clone()
                        } else {
                            print_error!(ExpectedToken, "expected identifier");
                        };

                        let mut args: Vec<TypedIdentifier> = Vec::new();
                        let mut return_type: Option<String> = None;
                        
                        if !is_token_of_type!(2, OpenParen) {
                            print_error!(ExpectedToken, "expected open paren '('");
                        }

                        let mut left_tokens = tokens.get(3..).unwrap().into_iter();
                        let mut redundant_pos: usize = 3;

                        macro_rules! next_token {
                            () => {{
                                redundant_pos += 1;
                                left_tokens.next()
                            }};
                        }

                        while let Some(token) = next_token!() {
                            match &token.kind {
                                TokenKind::ClosedParen => {
                                    // this returns function name:
                                    // if let Some(ident) = get_token_value!(pos + 1, Identifier) {
                                    //     return_type = Some(ident.clone());
                                    // }

                                    if let Some(token) = next_token!()
                                    && let TokenKind::Identifier(ident) = &token.kind {
                                        return_type = Some(ident.clone());
                                    }

                                    break;
                                }
                                TokenKind::DoubleIdentifier(ident, ty) => {
                                    let arg = TypedIdentifier {
                                        ident: ident.clone(),
                                        ty: ty.clone()
                                    };
                                    args.push(arg);
                                }
                                _ => {
                                    print_error!(ExpectedToken, "expected typed identifier `name:type`");
                                }
                            }
                        }

                        let return_type: String = if let Some(str) = return_type {
                            str
                        } else {
                            print_error!(ExpectedToken, "expected return type");
                        };

                        check_redundant_tokens!(redundant_pos - 1);

                        new_command = Some(Command {
                            kind: CommandKind::FunctionHeader { name, args, return_type }
                        });
                    }

                    "end" => {
                        if tokens.len() > 1 {
                            redundant_tokens_error!();
                        }

                        new_command = Some(Command {
                            kind: CommandKind::End
                        });
                    }

                    _ => {}
                }
            } else {
                print_error!(ExpectedToken, "expected identifier");
            }

            if let Some(command) = new_command && error.is_none() {
                commands.push(command);
            } else if error.is_some() {
                // free memory taken by commands (we won't return them anyway)
                commands = Vec::new();
            }
        }

        if let Some(error) = error {
            Err(error)
        } else {
            Ok(commands)
        }
    }
}

pub fn compile_from_file(file: File, filename: String) -> Result<Vec<u8>, ErrorKind> {
    let compiler = Parser::new(filename);

    let tokens = compiler.parse_file(file)?;

    for line in tokens.iter() {
        if let Some(tok) = line.first() {
            print!("[{}] ", tok.line_pos);
        } else {
            continue;
        }

        for token in line {
            print!("{:?}, ", token.kind);
        }

        println!();
    }

    let commands = compiler.parse_tokens(tokens)?;

    Ok(Vec::new())
}
