use std::fs::File;
use std::io::{BufRead, BufReader};

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

pub fn compile_from_file(file: File) -> Vec<u8> {
    let reader = BufReader::new(file);
    let input_lines = reader.lines().map(|line| line.unwrap());

    let mut lines: Vec<Vec<Token>> = Vec::new();
    
    for input_line in input_lines {
        println!(">>> {input_line}");

        let mut chars = input_line.chars().peekable();

        for ch in chars {
            
        }
    }

    Vec::new()
}
