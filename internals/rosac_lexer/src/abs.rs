//! Abstraction over the lexer to allow easy testing without performance overhead.
use rosa_errors::{Diag, DiagCtxt, DiagInner, RosaRes};

use crate::{
    tokens::{Token, TokenType},
    Lexer,
};

/// This is an abstraction over the [Lexer]
///
/// [Lexer]: crate::Lexer
pub trait AbsLexer {
    /// Return the next token, while advancing to the next token.
    /// If their is an error while lexing, it is emitted
    fn consume(&mut self) -> Option<Token>;

    /// Returns the next token without advancing to the next token.
    ///
    /// If we already reached the end of file, it will always return None.
    fn peek(&mut self) -> Option<&Token> {
        self.peek_nth(0)
    }

    /// Returns the nth token after the current one.
    ///
    /// If we already reached the end of file, it will always return None.
    fn peek_nth(&mut self, idx: usize) -> Option<&Token>;

    /// Did we reached the end of file?
    fn finished(&self) -> bool;
}

impl AbsLexer for Lexer<'_> {
    fn consume(&mut self) -> Option<Token> {
        match self.lex() {
            RosaRes::Good(tok) => Some(tok),
            RosaRes::Recovered(tok, errs) => {
                for err in errs {
                    err.emit();
                }

                Some(tok)
            }
            RosaRes::Unrecovered(err) => {
                err.emit();
                None
            }
        }
    }

    fn peek_nth(&mut self, _: usize) -> Option<&Token> {
        panic!("Cannot peek a token using this lexer. please use BufferedLexer if you want to.")
    }

    fn finished(&self) -> bool {
        self.idx >= self.file.filetext.len().into()
    }
}

pub const BUFFERED_LEXER_DEFAULT_CAPACITY: usize = 8;

pub struct BufferedLexer<'r> {
    /// The inner lexer, not able to peek tokens.
    inner: Lexer<'r>,
    /// The buffer containing pre-lexed tokens, used to be able to peek tokens
    /// when parsing.
    buf: Vec<Token>,
}

impl<'r> BufferedLexer<'r> {
    pub fn with_capacity(lexer: Lexer<'r>, cap: usize) -> BufferedLexer<'r> {
        BufferedLexer {
            inner: lexer,
            buf: Vec::with_capacity(cap),
        }
    }

    pub fn new(lexer: Lexer<'r>) -> BufferedLexer<'r> {
        Self::with_capacity(lexer, BUFFERED_LEXER_DEFAULT_CAPACITY)
    }

    pub fn pre_lex(&mut self, amount: usize) -> Vec<DiagInner> {
        let mut inner_diags = Vec::new();

        for i in 1..=amount {
            match self.inner.lex() {
                RosaRes::Good(tok) => {
                    if tok.tt == TokenType::EOF {
                        self.buf.push(tok);
                        break;
                    }
                    self.buf.push(tok);
                }
                RosaRes::Recovered(tok, errs) => {
                    for err in errs {
                        inner_diags.push(err.diag);
                    }

                    if tok.tt == TokenType::EOF {
                        self.buf.push(tok);
                        break;
                    }
                    self.buf.push(tok);
                }
                RosaRes::Unrecovered(err) => {
                    inner_diags.push(err.diag);
                }
            }
        }

        inner_diags
    }

    pub fn buf(&self) -> &[Token] {
        &self.buf
    }
}

impl<'r> AbsLexer for BufferedLexer<'r> {
    fn consume(&mut self) -> Option<Token> {
        if self.finished() {
            return None;
        }
        if self.buf.is_empty() {
            self.inner.consume()
        } else {
            let tok = self.buf.get(0)?.clone();
            self.buf.rotate_left(1);
            self.buf.truncate(self.buf.len() - 1);
            Some(tok)
        }
    }

    fn peek_nth(&mut self, idx: usize) -> Option<&Token> {
        if self.finished() {
            return None;
        }

        if idx + 1 > self.buf.len() {
            // the amount needed to pre lex
            let amount = idx - self.buf.len() + 1;
            for inner in self.pre_lex(amount) {
                Diag::from_inner(inner, self.inner.dcx).emit();
            }
        }

        self.buf.get(idx)
    }

    fn finished(&self) -> bool {
        self.inner.finished() && self.buf.is_empty()
    }
}
