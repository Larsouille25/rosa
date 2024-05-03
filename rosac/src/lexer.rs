//! Lexing of Rosa source code into Tokens.

use std::{fs::read_to_string, path::Path};

pub struct Lexer<'r> {
    filepath: &'r Path,
    filetext: String,
    prev_idx: usize,
    idx: usize,
}

impl<'r> Lexer<'r> {
    pub fn new(filepath: &Path, filetext: String) -> Lexer {
        Lexer {
            filepath,
            filetext,
            prev_idx: 0,
            idx: 0,
        }
    }

    pub fn from_filepath(filepath: &Path) -> std::io::Result<Lexer> {
        Ok(Lexer::new(filepath, read_to_string(filepath)?))
    }
}
