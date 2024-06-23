//! Rosa's tokens.

use std::{fmt::Display, str::FromStr};

use rosa_comm::Span;

#[derive(Debug, Clone)]
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
    NewLine,

    // End of file
    EOF,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KW(kw) => write!(f, "keyword `{kw}`"),
            Self::Punct(punct) => write!(f, "`{punct}`"),
            Self::Int(i) => write!(f, "int `{i}`"),
            Self::Str(s) => write!(f, "string {s:?}"),
            Self::Char(c) => write!(f, "char {c:?}"),
            Self::Ident(id) => write!(f, "identifier `{id}`"),
            Self::NewLine => write!(f, "new line"),
            Self::EOF => write!(f, "end of file"),
        }
    }
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

impl Display for Punctuation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::RParen => "(",
                Self::LParen => ")",

                Self::RBracket => "[",
                Self::LBracket => "]",

                Self::RBrace => "{",
                Self::LBrace => "}",

                Self::Colon => ":",
                Self::Semi => ";",
                Self::Comma => ",",
                Self::At => "@",

                Self::Asterisk => "*",
                Self::Caret => "^",
                Self::Dot => ".",
                Self::Equal => "=",
                Self::Equal2 => "==",
                Self::Exclamationmark => "!",
                Self::ExclamationmarkEqual => "!=",
                Self::LArrow => "<",
                Self::LArrow2 => "<<",
                Self::LArrowEqual => "<=",
                Self::Minus => "-",
                Self::Percent => "%",
                Self::Plus => "+",
                Self::RArrow => ">",
                Self::RArrow2 => ">>",
                Self::RArrowEqual => ">=",
                Self::Slash => "/",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    Fun,
    Return,
    Val,
    Var,
    Type,
    True,
    False,
    If,
    Else,
}

impl FromStr for Keyword {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "fun" => Keyword::Fun,
            "return" => Keyword::Return,
            "val" => Keyword::Val,
            "var" => Keyword::Var,
            "type" => Keyword::Type,
            "true" => Keyword::True,
            "false" => Keyword::False,
            "if" => Keyword::If,
            "else" => Keyword::Else,
            _ => return Err(()),
        })
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Fun => "fun",
                Self::Return => "return",
                Self::Val => "val",
                Self::Var => "var",
                Self::Type => "type",
                Self::True => "true",
                Self::False => "false",
                Self::If => "if",
                Self::Else => "else",
            }
        )
    }
}
