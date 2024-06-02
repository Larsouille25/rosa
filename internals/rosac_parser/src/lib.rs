use std::{fmt::Display, fmt::Write, marker::PhantomData};

use rosa_errors::DiagCtxt;
use rosac_lexer::{
    abs::{AbsLexer, BufferedLexer},
    tokens::{Keyword, Punctuation, Token, TokenType},
};

pub struct Parser<'r, L: AbsLexer = BufferedLexer<'r>> {
    lexer: L,
    // used to be able to make the L type default to BufferedLexer.
    _marker: PhantomData<&'r ()>,
}

impl<'r, L: AbsLexer> Parser<'r, L> {
    pub fn new(lexer: L) -> Parser<'r, L> {
        Parser {
            lexer,
            _marker: PhantomData,
        }
    }

    pub fn dcx(&self) -> &DiagCtxt {
        self.lexer.dcx()
    }

    /// Expects a token, emits a diag if the consumed token is not the expected one.
    ///
    /// Returns `Some(())` if everything went well and the token was the one.
    /// And `None` if the token wasn't the expected token.
    ///
    /// # Panic
    /// Panics if the lexer already reached the end of file.
    pub fn expect_token<Ex: Expectable>(&mut self, expect: Ex) -> Option<()> {
        match self.lexer.consume() {
            Some(Token { ref tt, .. }) if expect.answer(tt) => Some(()),
            Some(Token { tt, loc }) => {
                self.dcx().emit_diag(
                    self.dcx()
                        .struct_err(format!("expected {}, found {tt}", expect.format()), loc),
                );
                None
            }
            None if self.lexer.finished() => {
                panic!("Expect a token after the End Of File as been reached.")
            }
            None => None,
        }
    }
}

pub trait RosaParse {
    type Output;

    fn parse(parser: &mut Parser) -> Self::Output;
}

pub trait Expectable: Sized {
    fn answer(&self, tt: &TokenType) -> bool;

    fn format(&self) -> String;
}

pub enum TokenPattern {
    KW(Keyword),

    Punct(Punctuation),

    IntLiteral,
    StrLiteral,
    CharLiteral,

    Identifier,
    NamedIdentifier(String),

    Indent,
    NewLine,

    EndOfFile,
}

impl Display for TokenPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KW(kw) => write!(f, "{kw}"),
            Self::Punct(punct) => write!(f, "{punct}"),
            Self::IntLiteral => write!(f, "integer literal"),
            Self::StrLiteral => write!(f, "string literal"),
            Self::CharLiteral => write!(f, "char literal"),
            Self::Identifier => write!(f, "identifier"),
            Self::NamedIdentifier(name) => write!(f, "{name}"),
            Self::Indent => write!(f, "indendation"),
            Self::NewLine => write!(f, "new line"),
            Self::EndOfFile => write!(f, "end of file"),
        }
    }
}

impl Expectable for TokenPattern {
    fn answer(&self, tt: &TokenType) -> bool {
        use TokenPattern as TPat;
        use TokenType as TTy;
        match (self, tt) {
            (TPat::KW(kw1), TTy::KW(kw2)) if kw1 == kw2 => true,
            (TPat::Punct(punct1), TTy::Punct(punct2)) if punct1 == punct2 => true,
            (TPat::NamedIdentifier(i1), TTy::Ident(i2)) if i1 == i2 => true,
            (TPat::IntLiteral, TTy::Int(_))
            | (TPat::StrLiteral, TTy::Str(_))
            | (TPat::CharLiteral, TTy::Char(_))
            | (TPat::Identifier, TTy::Ident(_))
            | (TPat::Indent, TTy::Indent)
            | (TPat::NewLine, TTy::NewLine)
            | (TPat::EndOfFile, TTy::EOF) => true,
            _ => false,
        }
    }

    fn format(&self) -> String {
        self.to_string()
    }
}

impl<T: Expectable> Expectable for &[T] {
    fn answer(&self, tt: &TokenType) -> bool {
        for pat in self as &[T] {
            if pat.answer(tt) {
                return true;
            }
        }
        false
    }

    fn format(&self) -> String {
        match self.len() {
            0 => panic!("wtf"),
            1 => self[0].format(),
            2.. => {
                let mut s = String::new();
                for (idx, expect) in self.iter().enumerate() {
                    if idx == self.len() - 2 {
                        write!(s, "{} ", expect.format()).unwrap();
                    } else if idx == self.len() - 1 {
                        write!(s, "or {}", expect.format()).unwrap();
                    } else {
                        write!(s, "{}, ", expect.format()).unwrap();
                    }
                }
                s
            }
        }
    }
}
impl<T: Expectable, const N: usize> Expectable for [T; N] {
    fn answer(&self, tt: &TokenType) -> bool {
        (&self[..]).answer(tt)
    }

    fn format(&self) -> String {
        (&self[..]).format()
    }
}
