use std::{env, fs::read_to_string, path::PathBuf};

use termcolor::{ColorChoice, StandardStream};

use rosa_errors::DiagCtxt;
use rosac_lexer::{abs::BufferedLexer, Lexer};

use rosac_parser::Parser;

fn main() {
    println!("Hello, Rosa 🌹!\n");

    let args: Vec<String> = env::args().collect();
    assert_eq!(&args.len(), &2, "rosac <input file>");
    let path = PathBuf::from(&args[1]);
    let buf = read_to_string(&path).unwrap();
    let mut s = StandardStream::stdout(ColorChoice::Auto);

    let dcx = DiagCtxt::new(&buf, &path);

    let buf_lexer = BufferedLexer::new(Lexer::new(&path, &buf, &dcx));

    let mut parser = Parser::new(buf_lexer);

    dbg!(parser.begin_parsing());

    dcx.render_all(&mut s);
}
