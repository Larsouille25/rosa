use core::fmt;
use std::{fmt::Display, fmt::Write, marker::PhantomData};

use rosa_errors::{Diag, DiagCtxt, RosaRes};
use rosac_lexer::{
    abs::{AbsLexer, BufferedLexer},
    tokens::{Keyword, Punctuation, Token},
};

use crate::expr::Expression;

pub mod expr;

pub struct Parser<'r, L: AbsLexer = BufferedLexer<'r>> {
    lexer: L,
    // used to be able to make the L type default to BufferedLexer.
    _marker: PhantomData<&'r ()>,
}

// TODO: It is temporary, The top level ast node will not be expression.
type TopLevelAst = Expression;

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

    pub fn consume_tok(&mut self) -> Option<Token> {
        self.lexer.consume()
    }

    pub fn nth_tok(&mut self, idx: usize) -> Option<&Token> {
        self.lexer.peek_nth(idx)
    }

    pub fn peek_tok(&mut self) -> Option<&Token> {
        self.nth_tok(0)
    }

    pub fn begin_parsing(&mut self) -> TopLevelAst {
        dbg!(TopLevelAst::parse(self).unwrap());
        todo!()
    }
}

pub trait AstNode: fmt::Debug {
    type Output;

    fn parse<'r, L: AbsLexer>(parser: &'r mut Parser<'_, L>) -> RosaRes<Self::Output, Diag<'r>>;
}

pub enum FmtToken {
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

impl Display for FmtToken {
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

#[macro_export]
macro_rules! expect_token {
    ($parser:expr => [ $($token:pat, $result:expr);* ] else $unexpected:block) => (
        match $parser.peek_tok().unwrap().tt {
            $(
                $token => {
                    $parser.consume_tok();
                    $result
                }
            )*
            _ => $unexpected
        }
    );

    ($parser:expr => [ $($token:pat, $result:expr);* ], $expected:expr) => (
        $crate::expect_token!($parser => [ $($token, $result)* ] else {
            // TODO: maybe do some better error reporting
            let found = $parser.peek_tok().cloned().unwrap();
    //                     .struct_err(format!("expected {}, found {tt}", expect.format()), loc),
            return RosaRes::Unrecovered(
                $parser
                    .dcx()
                    .struct_err($crate::expected_tok_msg(found.tt, $expected), found.loc)
            );
        })
    )
}

fn expected_tok_msg<const N: usize>(found: impl Display, expected: [impl Display; N]) -> String {
    format!("expected {}, found {}", format_expected(expected), found)
}

fn format_expected<const N: usize>(exptd: [impl Display; N]) -> String {
    if exptd.len() == 1 {
        return format!("{}", exptd.first().unwrap());
    }
    let mut s = String::new();

    for (idx, token) in exptd.iter().enumerate() {
        if idx == exptd.len() - 2 {
            write!(s, "{token} ")
        } else if idx == exptd.len() - 1 {
            write!(s, "or {token}")
        } else {
            write!(s, "{token}, ")
        }
        .unwrap();
    }

    s
}
