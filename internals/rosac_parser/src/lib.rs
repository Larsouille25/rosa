use core::fmt;
use std::{
    fmt::{Display, Write},
    marker::PhantomData,
};

use precedence::PrecedenceValue;
use rosa_errors::{Diag, DiagCtxt, Fuzzy};
use rosac_lexer::{
    abs::{AbsLexer, BufferedLexer},
    tokens::{Keyword, Punctuation, Token},
};

use crate::expr::Expression;

pub mod expr;
pub mod precedence;

pub struct Parser<'r, L: AbsLexer = BufferedLexer<'r>> {
    /// the underlying lexer
    lexer: L,
    /// the actual precedence value when parsing expressions
    current_precedence: PrecedenceValue,
    /// used to be able to make the L type default to BufferedLexer.
    _marker: PhantomData<&'r ()>,
}

// TODO: It is temporary, The top level ast node will not be expression.
type TopLevelAst = Expression;

impl<'r, L: AbsLexer> Parser<'r, L> {
    pub fn new(lexer: L) -> Parser<'r, L> {
        Parser {
            lexer,
            current_precedence: 0,
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

    pub fn try_peek_tok(&mut self) -> Option<&Token> {
        self.nth_tok(0)
    }

    pub fn peek_tok(&mut self) -> &Token {
        self.try_peek_tok()
            .expect("Tried to peek a token after consuming the end of file token.")
    }

    pub fn begin_parsing(&mut self) -> Fuzzy<TopLevelAst, Diag> {
        TopLevelAst::parse(self)
    }
}

pub trait AstNode: fmt::Debug {
    type Output;

    fn parse<L: AbsLexer>(parser: &mut Parser<'_, L>) -> Fuzzy<Self::Output, Diag>;
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

pub enum AstNodes {
    Expression,
    Statement,
    FunctionDef,
    Definition,
    ImportDecl,
    UnaryOperator,
}

impl Display for AstNodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expression => write!(f, "expression"),
            Self::Statement => write!(f, "statement"),
            Self::FunctionDef => write!(f, "function definition"),
            Self::Definition => write!(f, "definition"),
            Self::ImportDecl => write!(f, "import declaration"),
            Self::UnaryOperator => write!(f, "unary operator"),
        }
    }
}

#[macro_export]
macro_rules! expect_token {
    ($parser:expr => [ $($token:pat, $result:expr);* ] else $unexpected:block) => (
        match &$parser.peek_tok().tt {
            $(
                $token => {
                    ($result, $parser.consume_tok().unwrap().loc)
                }
            )*
            _ => $unexpected
        }
    );

    ($parser:expr => [ $($token:pat, $result:expr);* ], $expected:expr) => (
        $crate::expect_token!($parser => [ $($token, $result)* ] else {
            let found = $parser.peek_tok().clone();
            return Fuzzy::Err(
                $parser
                    .dcx()
                    .struct_err($crate::expected_tok_msg(found.tt, $expected), found.loc)
            );
        })
    )
}

#[macro_export]
macro_rules! parse {
    ($parser:expr => $node:ty) => {
        parse!(fn; $parser => <$node as $crate::AstNode>::parse)
    };
    (fn; $parser:expr => $parsing_fn:expr $(, $arg:expr)*) => (
        match $parsing_fn($parser $(, $arg)*) {
            Fuzzy::Ok(ast) => ast,
            Fuzzy::Fuzzy(ast, diags) => {
                // for some obscur borrow checker reason we cannot use the
                // `emit_diags` method we are constrained to do it manually
                for diag in diags {
                    $parser.dcx().emit_diag(diag);
                }
                ast
            }
            Fuzzy::Err(err) => return Fuzzy::Err(err),
        }
    )
}

pub fn expected_tok_msg<const N: usize>(
    found: impl Display,
    expected: [impl Display; N],
) -> String {
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
