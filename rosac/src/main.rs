use std::{env, fs::read_to_string, path::PathBuf};

use termcolor::{ColorChoice, StandardStream};

use rosa_errors::{DiagCtxt, RosaRes};
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

    match parser.begin_parsing() {
        RosaRes::Good(ast) => {
            dbg!(ast);
        }
        RosaRes::Recovered(ast, diags) => {
            dcx.emit_diags(diags);
            dbg!(ast);
        }
        RosaRes::Unrecovered(diag) => dcx.emit_diag(diag),
    }

    dcx.render_all(&mut s);
}
