use core::fmt;
use std::{
    fmt::{Display, Write},
    marker::PhantomData,
};

use precedence::PrecedenceValue;
use rosa_comm::Span;
use rosa_errors::{Diag, DiagCtxt, Fuzzy};
use rosac_lexer::{
    abs::{AbsLexer, BufferedLexer},
    tokens::{Keyword, Punctuation, Token},
};
use stmt::Statement;

pub mod block;
pub mod expr;
pub mod precedence;
pub mod stmt;

pub struct Parser<'r, L: AbsLexer = BufferedLexer<'r>> {
    /// the underlying lexer
    lexer: L,
    /// the actual precedence value when parsing expressions
    current_precedence: PrecedenceValue,
    /// Current indentation level
    scopelvl: u32,
    /// How wide is an indentation
    indent_size: u32,
    /// used to be able to make the L type default to BufferedLexer.
    _marker: PhantomData<&'r ()>,
}

// TODO: It is temporary, The top level ast node will not be expression.
type TopLevelAst = Statement;

impl<'r, L: AbsLexer> Parser<'r, L> {
    pub fn new(lexer: L) -> Parser<'r, L> {
        Parser {
            lexer,
            current_precedence: 0,
            scopelvl: 0,
            indent_size: 0,
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
        self.try_peek_tok().unwrap()
    }

    pub fn begin_parsing(&mut self) -> Fuzzy<TopLevelAst, Diag> {
        TopLevelAst::parse(self)
    }

    pub fn enter_scope(&mut self, lf: &Span) {
        if self.scopelvl == 0 {
            self.indent_size = (self.peek_tok().loc.lo - lf.hi).0;
        }
        self.scopelvl += 1;
    }

    pub fn leave_scope(&mut self) {
        self.scopelvl -= 1;
    }

    pub fn scope(&mut self, lf: &Span) -> Option<u32> {
        let ws = self.try_peek_tok()?.loc.lo - lf.hi;
        Some(ws.0 / self.indent_size)
    }
}

pub trait AstNode: fmt::Debug {
    type Output: Location;

    fn parse<L: AbsLexer>(parser: &mut Parser<'_, L>) -> Fuzzy<Self::Output, Diag>;
}

pub trait Location {
    fn loc(&self) -> Span;
}

#[macro_export]
macro_rules! derive_loc {
    ($t:ty $(where $( $tt:tt )* )? ) => {
        impl $( $( $tt )* )?  $crate::Location for $t {
            fn loc(&self) -> rosa_comm::Span {
                self.loc.clone()
            }
        }
    };
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

pub enum AstPart {
    Expression,
    Statement,
    FunctionDef,
    Definition,
    ImportDecl,
    UnaryOperator,
}

impl Display for AstPart {
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
    ($parser:expr => [ $($token:pat, $result:expr $(,in $between:stmt)?);* ] else $unexpected:block) => (
        match &$parser.peek_tok().tt {
            $(
                $token => {
                    $(
                        $between
                    )?
                    #[allow(unreachable_code)]
                    // we allow unreacheable code because the $between type may be `!`
                    ($result, $parser.consume_tok().unwrap().loc)
                }
            )*
            _ => $unexpected
        }
    );

    ($parser:expr => [ $($token:pat, $result:expr $(,in $between:stmt)?);* ], $expected:expr) => (
        $crate::expect_token!($parser => [ $($token, $result $(, in $between)? );* ] else {
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
        parse!(@fn $parser => <$node as $crate::AstNode>::parse)
    };
    (@fn $parser:expr => $parsing_fn:expr $(, $arg:expr)*) => (
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
