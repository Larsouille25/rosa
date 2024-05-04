use std::{fs::read_to_string, path::PathBuf};

use rosac::lexer::Lexer;

fn main() {
    println!("Hello, Rosa 🌹!\n");

    // let path = PathBuf::from("examples/fib.ro");
    let path = PathBuf::from("test.ro");
    let buf = read_to_string(&path).unwrap();

    let mut lexer = Lexer::new(&path, &buf);
    let tokens = lexer.lex();
    let _ = dbg!(tokens);
}
