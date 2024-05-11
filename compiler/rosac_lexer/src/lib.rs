//! Lexing of Rosa source code into Tokens.

use std::{iter::Peekable, ops::Range, path::Path, str::Chars};

use crate::tokens::{Token, TokenType};

use crate::tokens::TokenType::*;
use crate::tokens::{Keyword, Punctuation};

pub mod tokens;

pub struct LexrFile<'r> {
    filepath: &'r Path,
    filetext: &'r str,
    /// Index of the last `pop`ed char, starting from 1.
    idx: usize,

    iter: Peekable<Chars<'r>>,
}

impl<'r> LexrFile<'r> {
    pub fn pop(&mut self) -> Option<char> {
        self.idx += 1;
        self.iter.next()
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.iter.peek()
    }

    pub fn filepath(&self) -> &'r Path {
        self.filepath
    }

    pub fn filetext(&self) -> &'r str {
        self.filetext
    }

    /// NOTE: This function can slow the lexing, it shouldn't be called too
    /// often.
    pub fn reset(&mut self) {
        self.iter = self.filetext.chars().peekable();
        self.idx = 0;
    }

    /// Returns the true length, the count of how many Unicode characters there is
    /// in the source code file.
    pub fn length(&self) -> usize {
        // NOTE: This function is slow because it creates a new iterator each
        // time it's called, if it's called to much time, we should consider
        // storing the lenght of the file in a field and compute it only once.
        self.filetext.chars().count()
    }

    /// Resets the iterator and put the iterator to the new index. The index
    /// starts from 1.
    ///
    /// NOTE: This function can slow the lexing, it shouldn't be called too
    /// often.
    pub fn reset_to(&mut self, new_idx: usize) -> Option<()> {
        if new_idx > self.length() {
            return None;
        }
        self.reset();
        // TODO: use `advance_by` method on the iterator when it will be
        // stabilized
        for _ in 0..new_idx {
            if self.iter.next().is_none() {
                unreachable!("Should've been caught before.")
            }
        }
        self.idx = new_idx;

        Some(())
    }

    pub fn relative_reset(&mut self, offset: isize) -> Option<()> {
        let new_idx = ((self.idx as isize + offset).try_into() as Result<usize, _>).ok()?;
        self.reset_to(new_idx)
    }
}

pub enum PartTokenResult {
    Tok(TokenType),
    Error(String),
    PartOk(TokenType, Vec<String>),
    Comment,
    OtherWS,
}

use PartTokenResult::*;

pub struct Lexer<'r> {
    file: LexrFile<'r>,
    prev_idx: usize,
    idx: usize,
}

impl<'r> Lexer<'r> {
    pub fn new(filepath: &'r Path, filetext: &'r str) -> Lexer<'r> {
        Lexer {
            file: LexrFile {
                filepath,
                filetext,
                idx: 0,
                iter: filetext.chars().peekable(),
            },
            prev_idx: 0,
            idx: 0,
        }
    }

    pub fn pop(&mut self) -> Option<char> {
        self.idx += 1;
        self.file.pop()
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.file.peek()
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, ()> {
        let mut tokens = Vec::new();

        loop {
            self.prev_idx = self.idx;
            match self.make_token() {
                Tok(tt) => {}
                Error(err) => {
                    println!("{}", err);
                    return Err(());
                }
                PartOk(tt, errs) => {}
                Comment | OtherWS => {}
            }
        }

        Ok(tokens)
    }

    pub fn make_token(&mut self) -> PartTokenResult {
        let t = match self.peek() {
            Some('A'..='Z' | 'a'..='z' | '_' | '0'..='9') => {
                todo!("We've got an indentifier, keyword or integer literal!")
            }
            Some(c) => return Error(format!("unknown start of token {:?}", c)),
            None => EOF,
        };
        self.idx += 1;
        Tok(t)
    }
}