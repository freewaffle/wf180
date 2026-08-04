use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn compile_from_file(file: File) -> Vec<u8> {
    let mut reader = BufReader::new(file);
    let lines = reader.lines().map(|line| line.unwrap());
    
    for line in lines {
        println!("{line}");
    }

    Vec::new()
}
