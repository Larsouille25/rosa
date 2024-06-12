use rosa_comm::Span;
use rosa_errors::Diag;
use rosac_lexer::{abs::AbsLexer, tokens::TokenType};

use crate::{expect_token, AstNode, FmtToken, Parser, RosaRes};

#[derive(Debug)]
pub struct Expression {
    expr: ExpressionInner,
    loc: Span,
}

#[derive(Debug)]
pub enum ExpressionInner {
    IntLiteral(u64),
}

impl AstNode for Expression {
    type Output = Self;

    fn parse<'a, L: AbsLexer>(parser: &'a mut Parser<'_, L>) -> RosaRes<Self::Output, Diag<'a>> {
        println!();
        dbg!(expect_token!(parser => [TokenType::Int(i), i], [FmtToken::IntLiteral]));
        println!();
        todo!()
    }
}
