//! Module responsible for parsing Statements

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Statement {
    pub stmt: StatementInner,
    pub loc: Span,
}

derive_loc!(Statement);

impl AstNode for Statement {
    type Output = Self;

    fn parse<L: AbsLexer>(parser: &mut Parser<'_, L>) -> Fuzzy<Self::Output, Diag> {
        StatementInner::parse(parser)
    }
}

#[derive(Debug, Clone)]
pub enum StatementInner {
    IfStmt {
        predicate: Expression,
        body: Block<Statement>,
        else_branch: Option<Block<Statement>>,
    },
    ExprStmt(Expression),
    ReturnStmt(Option<Expression>),
}

impl AstNode for StatementInner {
    type Output = Statement;

    fn parse<L: AbsLexer>(parser: &mut Parser<'_, L>) -> Fuzzy<Self::Output, Diag> {
        match parser.peek_tok() {
            Token {
                tt: KW(Keyword::If),
                ..
            } => parse_if_stmt(parser),
            Token {
                tt: KW(Keyword::Return),
                ..
            } => parse_return_stmt(parser),
            _ => parse_expr_stmt(parser),
        }
    }
}

pub fn parse_expr_stmt(parser: &mut Parser<'_, impl AbsLexer>) -> Fuzzy<Statement, Diag> {
    let expr = parse!(parser => Expression);
    // TODO: try to improve the errors, here when parsing of expression fails
    // it says 'expected expression, found ..'
    Fuzzy::Ok(Statement {
        loc: expr.loc.clone(),
        stmt: StatementInner::ExprStmt(expr),
    })
}

pub fn parse_if_stmt(parser: &mut Parser<'_, impl AbsLexer>) -> Fuzzy<Statement, Diag> {
    let (_, Span { lo, .. }) =
        expect_token!(parser => [KW(Keyword::If), ()], [FmtToken::KW(Keyword::If)]);
    let predicate = parse!(parser => Expression);

    expect_token!(
        parser => [Punct(Punctuation::Colon), ()],
        [FmtToken::Punct(Punctuation::Colon)]
    );
    let body = parse!(parser => Block<Statement>);
    let mut hi = body.loc.hi;

    let else_branch = if let Some(Token {
        tt: KW(Keyword::Else),
        ..
    }) = parser.try_peek_tok()
    {
        expect_token!(parser => [KW(Keyword::Else), ()], [FmtToken::KW(Keyword::Else)]);

        expect_token!(
            parser => [Punct(Punctuation::Colon), ()],
            [FmtToken::Punct(Punctuation::Colon)]
        );

        let r#else = parse!(parser => Block<Statement>);
        hi = r#else.loc.hi;
        Some(r#else)
    } else {
        None
    };

    Fuzzy::Ok(Statement {
        stmt: StatementInner::IfStmt {
            predicate,
            body,
            else_branch,
        },
        loc: Span::new(lo, hi),
    })
}

pub fn parse_return_stmt(parser: &mut Parser<'_, impl AbsLexer>) -> Fuzzy<Statement, Diag> {
    let ((), mut loc) =
        expect_token!(parser => [KW(Keyword::Return), ()], [FmtToken::KW(Keyword::Return)]);
    dbg!(parser.try_peek_tok());

    if let Some(NewLine) = parser.try_peek_tok().map(|t| t.tt.clone()) {
        return Fuzzy::Ok(Statement {
            stmt: StatementInner::ReturnStmt(None),
            loc,
        });
    }

    let expr = parse!(parser => Expression);
    loc = Span::from_ends(loc, expr.loc.clone());

    Fuzzy::Ok(Statement {
        stmt: StatementInner::ReturnStmt(Some(expr)),
        loc,
    })
}
