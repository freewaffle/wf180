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
    let lines = reader.lines().map(|line| line.unwrap());
    
    for line in lines {
        println!(">>> {line}");
    }

    Vec::new()
}
