use rosa_comm::Span;
use rosa_errors::{
    Diag,
    RosaRes::{self, *},
};
use rosac_lexer::{
    abs::AbsLexer,
    tokens::{Token, TokenType::*},
};

use crate::{expect_token, expected_tok_msg, AstNode, FmtToken, Parser};

#[derive(Debug)]
pub struct Expression {
    pub expr: ExpressionInner,
    pub loc: Span,
}

#[derive(Debug)]
pub enum ExpressionInner {
    IntLiteral(u64),
}

impl AstNode for Expression {
    type Output = Self;

    fn parse<'a, L: AbsLexer>(parser: &'a mut Parser<'_, L>) -> RosaRes<Self::Output, Diag<'a>> {
        match parser.peek_tok() {
            Token { tt: Int(_), .. } => parse_intlit_expr(parser),
            t => {
                let t = t.clone();
                Unrecovered(
                    parser
                        .dcx()
                        .struct_err(expected_tok_msg(t.tt, [FmtToken::IntLiteral]), t.loc),
                )
            }
        }
    }
}

pub fn parse_intlit_expr<'a>(
    parser: &'a mut Parser<'_, impl AbsLexer>,
) -> RosaRes<Expression, Diag<'a>> {
    let (i, loc) = expect_token!(parser => [Int(i), i], [FmtToken::IntLiteral]);
    Good(Expression {
        expr: ExpressionInner::IntLiteral(i),
        loc,
    })
}
