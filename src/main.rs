#![deny(clippy::all)]
#![forbid(unsafe_code)]

mod magic;
mod compiler;

use std::fs::File;
use std::io::{Read, Seek};

use magic::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    let path = if let Some(path) = args.get(1) {
        path
    } else {
        eprintln!("error: missing file name");
        eprintln!("try executing: wf180 [FILENAME]");
        std::process::exit(1);
    };

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("error: can't open [{path}]: {}", err.kind());
            std::process::exit(1);
        }
    };

    let mut this_magic = [0u8; 6];
    let bytes_read = match file.read(&mut this_magic) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("error: can't read [{path}]: {}", err.kind());
            std::process::exit(1);
        }
    };

    if bytes_read < MAGIC_LEN {
        eprintln!("error: [{path}] is too short");
        std::process::exit(1);
    }

    let code: Vec<u8> = if this_magic == MAGIC {
        let mut code: Vec<u8> = Vec::new();
        if let Err(err) = file.read_to_end(&mut code) {
            eprintln!("error: can't read [{path}]: {}", err.kind());
            std::process::exit(1);
        } else {
            code
        }
    } else {
        println!("compiling...");
        file.rewind().unwrap();
        compiler::compile_from_file(file)
    };
}

/* use raylib::prelude::*;

fn main() {
    let (mut rl, rl_thread) = raylib::init()
        .size(640, 480)
        .title("WF-180")
        .build();

    while !rl.window_should_close() {
        rl.draw(&rl_thread, | mut drawer | {
            drawer.clear_background(Color::WHITE);
            drawer.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
        });
    }
} */
