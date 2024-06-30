use core::fmt;
use std::{
    fmt::{Display, Write},
    marker::PhantomData,
};

use decl::Declaration;
use precedence::PrecedenceValue;
use rosa_comm::{BytePos, Span};
use rosa_errors::{Diag, DiagCtxt, Fuzzy};
use rosac_lexer::{
    abs::{AbsLexer, BufferedLexer},
    tokens::{Keyword, Punctuation, Token, TokenType},
};

pub mod block;
pub mod decl;
pub mod expr;
pub mod precedence;
pub mod stmt;
pub mod types;

pub struct Parser<'r, L: AbsLexer = BufferedLexer<'r>> {
    /// the underlying lexer
    lexer: L,
    /// the actual precedence value when parsing expressions
    current_precedence: PrecedenceValue,
    /// Indent stack.
    indent: Vec<BytePos>,
    /// used to be able to make the L type default to BufferedLexer.
    _marker: PhantomData<&'r ()>,
}

impl<'r, L: AbsLexer> Parser<'r, L> {
    pub fn new(lexer: L) -> Parser<'r, L> {
        Parser {
            lexer,
            current_precedence: 0,
            indent: vec![0.into()],
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn dcx(&self) -> &DiagCtxt {
        self.lexer.dcx()
    }

    #[inline]
    pub fn consume_tok(&mut self) -> Option<Token> {
        self.lexer.consume()
    }

    #[inline]
    pub fn nth_tok(&mut self, idx: usize) -> Option<&Token> {
        self.lexer.peek_nth(idx)
    }

    #[inline]
    pub fn try_peek_tok(&mut self) -> Option<&Token> {
        self.nth_tok(0)
    }

    #[inline]
    pub fn peek_tok(&mut self) -> &Token {
        self.try_peek_tok().unwrap()
    }

    pub fn begin_parsing(&mut self) -> Vec<Declaration> {
        // TODO: maybe replace this with a 'Block<Declaration>'
        let mut decls = Vec::new();

        loop {
            while let Some(Token {
                tt: TokenType::NewLine,
                ..
            }) = self.try_peek_tok()
            {
                self.consume_tok();
            }

            if let Some(Token {
                tt: TokenType::EOF, ..
            }) = self.try_peek_tok()
            {
                break;
            }

            match Declaration::parse(self) {
                Fuzzy::Ok(decl) => decls.push(decl),
                Fuzzy::Fuzzy(decl, diags) => {
                    decls.push(decl);
                    self.dcx().emit_diags(diags);
                }
                Fuzzy::Err(diag) => {
                    // Here we break out of the loop because we didn't have a thing that
                    // correctly parses...
                    self.dcx().emit_diag(diag);
                    break;
                }
            }
        }

        decls
    }

    pub fn indent(&mut self, size: BytePos) {
        self.indent.push(size);
    }

    pub fn dedent(&mut self) -> Option<BytePos> {
        self.indent.pop()
    }

    pub fn last_indent(&self) -> Option<BytePos> {
        self.indent.last().copied()
    }

    /// Compute the indentation of the next token that is not a 'NewLine'
    pub fn compute_indent(&mut self) -> Option<(BytePos, usize)> {
        let lf = self.try_peek_tok()?.loc.clone();

        let (mut idx, mut ws) = (1, BytePos(0));
        while let Some(Token {
            tt: TokenType::NewLine,
            loc,
        }) = self.nth_tok(idx)
        {
            idx += 1;
            ws = loc.hi - lf.hi;
        }

        let next = self.nth_tok(idx)?.loc.clone();

        let gap = next.lo - lf.hi - ws;
        Some((gap, idx))
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
        impl $( $( $tt )* )? $crate::Location for $t {
            #[inline]
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
    Declaration,
    ImportDecl,
    UnaryOperator,
    Type,
}

impl Display for AstPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expression => write!(f, "expression"),
            Self::Statement => write!(f, "statement"),
            Self::FunctionDef => write!(f, "function definition"),
            Self::Declaration => write!(f, "declaration"),
            Self::ImportDecl => write!(f, "import declaration"),
            Self::UnaryOperator => write!(f, "unary operator"),
            Self::Type => write!(f, "type"),
        }
    }
}

#[macro_export]
macro_rules! expect_token {
    ($parser:expr => [ $($token:pat, $result:expr $(,in $between:stmt)?);* ] else $unexpected:block) => (
        // TODO: try to use 'try_peek_tok' instead, because it could panic
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
