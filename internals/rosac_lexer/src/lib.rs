//! Lexing of Rosa source code into Tokens.

use std::str::{CharIndices, FromStr};
use std::{iter::Peekable, path::Path};

use crate::tokens::{Token, TokenType};

use crate::tokens::TokenType::*;
use crate::tokens::{Keyword, Punctuation};
use rosa_comm::{BytePos, Span};
use rosa_errors::DiagCtxt;
use rosa_errors::{
    Diag,
    RosaRes::{self, *},
};

pub mod abs;
pub mod literals;
pub mod tokens;

pub struct LexrFile<'r> {
    filepath: &'r Path,
    filetext: &'r str,
    /// Index of the last `pop`ed char, starting from 1.
    idx: BytePos,

    iter: Peekable<CharIndices<'r>>,
}

impl<'r> LexrFile<'r> {
    pub fn pop(&mut self) -> Option<char> {
        let (i, ch) = self.iter.next()?;
        self.idx = i.into();
        Some(ch)
    }

    pub fn peek(&mut self) -> Option<char> {
        Some(self.iter.peek()?.1)
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
        self.iter = self.filetext.char_indices().peekable();
        self.idx = BytePos::ZERO;
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
            if let Some((i, _)) = self.iter.next() {
                self.idx = i.into();
            } else {
                unreachable!("Should've been caught before.")
            }
        }

        Some(())
    }

    pub fn relative_reset(&mut self, offset: isize) -> Option<()> {
        let new_idx = ((<BytePos as Into<usize>>::into(self.idx) as isize + offset).try_into()
            as Result<usize, _>)
            .ok()?;
        self.reset_to(new_idx)
    }
}

pub struct Lexer<'r> {
    file: LexrFile<'r>,
    prev_idx: BytePos,
    idx: BytePos,
    dcx: &'r DiagCtxt<'r>,
}

impl<'r> Lexer<'r> {
    pub fn new(filepath: &'r Path, filetext: &'r str, dcx: &'r DiagCtxt<'r>) -> Lexer<'r> {
        Lexer {
            file: LexrFile {
                filepath,
                filetext,
                idx: 0.into(),
                iter: filetext.char_indices().peekable(),
            },
            prev_idx: 0.into(),
            idx: 0.into(),
            dcx,
        }
    }

    /// Advance the iterator and the index (self.idx)
    pub fn pop(&mut self) -> Option<char> {
        self.idx += 1;
        self.file.pop()
    }

    /// Advance the underlying iterator without returning the result.
    pub fn advance(&mut self) {
        self.file.pop();
    }

    pub fn peek(&mut self) -> Option<char> {
        self.file.peek()
    }

    pub fn current_span(&self) -> Span {
        Span {
            lo: self.prev_idx,
            hi: self.idx,
        }
    }

    pub fn window(&self, size: usize) -> Option<&str> {
        self.file
            .filetext
            .get(self.prev_idx.0 as usize..self.prev_idx.0 as usize + size)
    }

    pub fn lex(&mut self) -> RosaRes<Token, Diag<'_>> {
        self.skip_useless_whitespace();

        self.prev_idx = self.idx;

        let tt = 'm: {
            match self.pop() {
                Some(c @ ('A'..='Z' | 'a'..='z' | '_' | '0'..='9')) => {
                    return self.lex_word(c);
                }
                Some('\n') => NewLine,
                // an indentation is either 4 spaces or a tabulation.
                Some(' ') if self.window(4) == Some("    ") => {
                    self.pop();
                    self.pop();
                    self.pop();
                    Indent
                }
                Some('\t') => Indent,
                Some(c) => {
                    if let Some(punct) = self.could_make_punct(c) {
                        // pop the lenght of the punctuation.
                        for _ in 0..punct.size() - 1 {
                            self.pop();
                        }
                        break 'm TokenType::Punct(punct);
                    }
                    let err = self
                        .dcx
                        .struct_err(format!("unknown start of token {c:?}"), self.current_span());
                    return Unrecovered(err);
                }
                None => EOF,
            }
        };

        Good(Token {
            tt,
            loc: self.current_span(),
        })
    }

    pub(crate) fn make_word(&mut self, c: char) -> (String, bool) {
        let mut word = String::from(c);
        let mut numeric = true;

        while let Some(c) = self.peek() {
            match c {
                'A'..='Z' | 'a'..='z' => {
                    word.push(c);
                    numeric = false;
                }
                '0'..='9' | '_' => {
                    word.push(c);
                }
                _ => break,
            }
            self.pop();
        }

        (word, numeric)
    }

    pub(crate) fn lex_word(&mut self, c: char) -> RosaRes<Token, Diag<'_>> {
        let (word, numeric) = self.make_word(c);

        let tt = if numeric {
            return self.lex_int(word);
        } else {
            self.lex_keyword(word)
        };

        Good(Token {
            tt,
            loc: self.current_span(),
        })
    }

    pub(crate) fn lex_keyword(&self, word: String) -> TokenType {
        if let Ok(kw) = Keyword::from_str(&word) {
            TokenType::KW(kw)
        } else {
            TokenType::Ident(word)
        }
    }

    pub(crate) fn skip_useless_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                ' ' if self.window(4) != Some("    ") => {
                    self.pop();
                }
                '\u{000B}'..='\u{000D}'
                | '\u{0085}'
                | '\u{00A0}'
                | '\u{1680}'
                | '\u{2000}'..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}' => {
                    self.pop();
                }
                _ => break,
            }
        }
    }

    pub(crate) fn could_make_punct(&mut self, c: char) -> Option<Punctuation> {
        use Punctuation::*;
        Some(match c {
            // single char punctuation
            '(' => RParen,
            ')' => LParen,
            '[' => RBracket,
            ']' => LBracket,
            '{' => RBrace,
            '}' => LBrace,
            ':' => Colon,
            ';' => Semi,
            ',' => Comma,
            '@' => At,
            '*' => Asterisk,
            '^' => Caret,
            '.' => Dot,
            '-' => Minus,
            '%' => Percent,
            '+' => Plus,
            '/' => Slash,

            // ambigious
            '!' => match self.peek() {
                Some('=') => ExclamationmarkEqual,
                _ => Exclamationmark,
            },
            '=' => match self.peek() {
                Some('=') => Equal2,
                _ => Equal,
            },
            '<' => match self.peek() {
                Some('<') => LArrow2,
                Some('=') => LArrowEqual,
                _ => LArrow,
            },
            '>' => match self.peek() {
                Some('>') => RArrow2,
                Some('=') => RArrowEqual,
                _ => RArrow,
            },

            _ => return None,
        })
    }
}
