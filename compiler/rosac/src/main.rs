use std::{env, fs::read_to_string, path::PathBuf};

use termcolor::{ColorChoice, StandardStream};

use rosa_comm::Span;
use rosa_errors::DiagCtxt;
use rosac_lexer::Lexer;

fn main() {
    println!("Hello, Rosa 🌹!\n");

    let args: Vec<String> = env::args().collect();
    assert_eq!(&args.len(), &2, "rosac <input file>");
    let path = PathBuf::from(&args[1]);
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
    dcx.push_diag(err);
    dcx.emit_all(&mut s);

    let mut lexer = Lexer::new(&path, &buf);
    let tokens = lexer.lex();
    let _ = dbg!(tokens);
}
