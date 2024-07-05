use std::{env, fs::read_to_string, path::PathBuf};

use rosac_sema::SemanticAnalyzer;
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

    let mut ast = parser.begin_parsing();

    let mut seman = SemanticAnalyzer::new(&mut ast, &dcx);
    dcx.emit_diags(seman.analyze());
    dbg!(&ast);

    dcx.render_all(&mut s);
}
