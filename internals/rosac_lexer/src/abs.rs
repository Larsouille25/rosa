//! Abstraction over the lexer to allow easy testing without performance overhead.
use crate::prelude::*;

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

    /// Get the Diag Context.
    fn dcx(&self) -> &DiagCtxt;
}

impl AbsLexer for Lexer<'_> {
    fn consume(&mut self) -> Option<Token> {
        match self.lex() {
            Fuzzy::Ok(tok) => Some(tok),
            Fuzzy::Fuzzy(tok, diags) => {
                for diag in diags {
                    self.dcx().emit_diag(diag)
                }

                Some(tok)
            }
            Fuzzy::Err(diag) => {
                self.dcx().emit_diag(diag);
                None
            }
        }
    }

    fn peek_nth(&mut self, _: usize) -> Option<&Token> {
        panic!("Cannot peek a token using this lexer. please use BufferedLexer if you want to.")
    }

    fn finished(&self) -> bool {
        self.idx > self.file.filetext.len().into()
    }

    fn dcx(&self) -> &DiagCtxt {
        self.dcx
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

    pub fn pre_lex(&mut self, amount: usize) -> Vec<Diag> {
        let mut inner_diags = Vec::new();

        for _ in 1..=amount {
            match self.inner.lex() {
                Fuzzy::Ok(tok) => {
                    if tok.tt == TokenType::EOF {
                        self.buf.push(tok);
                        break;
                    }
                    self.buf.push(tok);
                }
                Fuzzy::Fuzzy(tok, diags) => {
                    for diag in diags {
                        inner_diags.push(diag);
                    }

                    if tok.tt == TokenType::EOF {
                        self.buf.push(tok);
                        break;
                    }
                    self.buf.push(tok);
                }
                Fuzzy::Err(diag) => {
                    inner_diags.push(diag);
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
            let tok = self.buf.first()?.clone();
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
            let mut lexed = 0;
            let initial_len = self.buf.len();

            // we loop until we have enough tokens or we reached the end of file
            loop {
                let res = self.pre_lex(amount);
                for diag in &res {
                    self.dcx().emit_diag(diag.clone());
                }
                lexed += self.buf.len() - initial_len;
                if (lexed >= amount) || self.finished() {
                    break;
                }
            }
        }

        self.buf.get(idx)
    }

    fn finished(&self) -> bool {
        self.inner.finished() && self.buf.is_empty()
    }

    fn dcx(&self) -> &DiagCtxt {
        self.inner.dcx()
    }
}
