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

    let dcx = DiagCtxt::new(&buf, &path);
    let err = dcx.struct_spans_err(
        "test",
        vec![Span {
            lo: 0.into(),
            hi: 12.into(),
        }],
    );
    err.format(&mut s).unwrap();

    let mut lexer = Lexer::new(&path, &buf);
    let tokens = lexer.lex();
    let _ = dbg!(tokens);
}
