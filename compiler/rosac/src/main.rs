use std::{env, fs::read_to_string, path::PathBuf};

use termcolor::{ColorChoice, StandardStream};

use rosa_errors::{DiagCtxt, RecoverableRes::*};
use rosac_lexer::{tokens::TokenType, Lexer};

fn main() {
    println!("Hello, Rosa 🌹!\n");

    let args: Vec<String> = env::args().collect();
    assert_eq!(&args.len(), &2, "rosac <input file>");
    let path = PathBuf::from(&args[1]);
    let buf = read_to_string(&path).unwrap();
    let mut s = StandardStream::stdout(ColorChoice::Auto);

    let dcx = DiagCtxt::new(&buf, &path);

    let mut lexer = Lexer::new(&path, &buf, &dcx);

    loop {
        let res = lexer.lex();

        match res {
            Good(tok) => {
                dbg!(&tok);
                if tok.tt == TokenType::EOF {
                    break;
                }
            }
            Recovered(tok, err) => {
                dbg!(&tok);
                if tok.tt == TokenType::EOF {
                    break;
                }
                err.emit();
            }
            Unrecovered(err) => {
                err.emit();
            }
        }
    }
    dcx.render_all(&mut s);
}
