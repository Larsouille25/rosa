//! Rosa's tokens.

use std::str::FromStr;

use rosa_comm::Span;

#[derive(Debug)]
pub struct Token {
    pub tt: TokenType,
    pub loc: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Keywords
    KW(Keyword),

    // Operators and Punctuation
    Punct(Punctuation),

    // Literals
    Int(u64),
    Str(String),
    Char(char),

    Ident(String),

    // Special White Space
    Indent,
    NewLine,

    // End of file
    EOF,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Punctuation {
    // Delimiters:
    RParen,
    LParen,

    RBracket,
    LBracket,

    RBrace,
    LBrace,

    // Punctuation:
    Colon,
    Semi,
    Comma,
    At,

    // Operators:
    Asterisk,
    Caret,
    Dot,
    Equal,
    Equal2,
    Exclamationmark,
    ExclamationmarkEqual,
    LArrow,
    LArrow2,
    LArrowEqual,
    Minus,
    Percent,
    Plus,
    RArrow,
    RArrow2,
    RArrowEqual,
    Slash,
}

impl Punctuation {
    pub fn size(&self) -> usize {
        use Punctuation::*;
        match self {
            RParen | LParen | RBracket | LBracket | RBrace | LBrace | Colon | Semi | Comma | At
            | Asterisk | Caret | Dot | Equal | Exclamationmark | LArrow | Minus | Percent
            | Plus | RArrow | Slash => 1,
            Equal2 | ExclamationmarkEqual | LArrow2 | LArrowEqual | RArrow2 | RArrowEqual => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    Fun,
    Ret,
    Val,
    Var,
    Type,
    True,
    False,
}

impl FromStr for Keyword {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "fun" => Keyword::Fun,
            "ret" => Keyword::Ret,
            "val" => Keyword::Val,
            "var" => Keyword::Var,
            "type" => Keyword::Type,
            "true" => Keyword::True,
            "false" => Keyword::False,
            _ => return Err(()),
        })
    }
}
