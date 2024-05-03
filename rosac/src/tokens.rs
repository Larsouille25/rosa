//! Rosa's tokens.

use std::ops::Range;

pub struct Token<'r> {
    pub tt: TokenType,
    pub lexeme: &'r str,
    pub loc: Range<usize>,
}

pub enum TokenType {
    KW(Keyword),
    Punct(Punctuation),

    Int(u64),
    Str(String),
    Char(char),

    Ident(String),

    EOF,
}

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
    SemiColon,
    Comma,
    At,

    // Operators:
    Asterisk,
    Caret,
    Dot,
    Equal,
    Equal2,
    Exclamationmark,
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

pub enum Keyword {
    Fun,
    Ret,
    Val,
    Var,
    Type,
    True,
    False,
}
