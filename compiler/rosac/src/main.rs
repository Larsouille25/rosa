use std::{fs::read_to_string, path::PathBuf};

use termcolor::{ColorChoice, StandardStream};

use rosa_comm::Span;
use rosa_errors::DiagCtxt;
use rosac_lexer::Lexer;

fn main() {
    println!("Hello, Rosa 🌹!\n");

    // let path = PathBuf::from("examples/fib.ro");
    let path = PathBuf::from("test.ro");
    let buf = read_to_string(&path).unwrap();
    let mut s = StandardStream::stdout(ColorChoice::Auto);

    let mut dcx = DiagCtxt::new(&buf, &path, &mut s);
    let err = dcx.diag_err("test", Span { lo: 1, hi: 2 });
    // dcx.render_diag(&err); // <-- ERROR, can't borrow twice dcx as mutable.

    let mut lexer = Lexer::new(&path, &buf);
    let tokens = lexer.lex();
    let _ = dbg!(tokens);
}
