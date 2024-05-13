//! Rosa's tokens.

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
