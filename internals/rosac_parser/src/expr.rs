use rosa_comm::Span;
use rosa_errors::{
    Diag,
    RosaRes::{self, *},
};
use rosac_lexer::{
    abs::AbsLexer,
    tokens::{Punctuation, Token, TokenType::*},
};

use crate::{expect_token, expected_tok_msg, parse, AstNode, FmtToken, Parser};

/// Binary Operators
pub enum BinaryOp {
    /// Multiplication
    Mul,
    /// Division
    Div,
    /// Remainder
    Rem,
    /// Addition
    Add,
    /// Substraction
    Sub,
    /// Right shift
    RShift,
    /// Left shift
    LShift,
    /// Comparison Less Than
    CompLT,
    /// Comparison Greater Than
    CompGT,
    /// Comparison Less Than or Equal
    CompLTE,
    /// Comparison Greater Than or Equal
    CompGTE,
    /// Comparison Equal
    CompEq,
    /// Comparison Not Equal
    CompNe,
}

impl BinaryOp {
    pub fn from_punct(punct: Punctuation) -> Option<BinaryOp> {
        use BinaryOp as BOp;
        use Punctuation as Punct;
        Some(match punct {
            Punct::Asterisk => BOp::Mul,
            Punct::Slash => BOp::Div,
            Punct::Percent => BOp::Rem,
            Punct::Plus => BOp::Add,
            Punct::Minus => BOp::Sub,
            Punct::RArrow2 => BOp::RShift,
            Punct::LArrow2 => BOp::LShift,
            Punct::LArrow => BOp::CompLT,
            Punct::RArrow => BOp::CompGT,
            Punct::LArrowEqual => BOp::CompLTE,
            Punct::RArrowEqual => BOp::CompGTE,
            Punct::Equal2 => BOp::CompEq,
            Punct::ExclamationmarkEqual => BOp::CompNe,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub expr: ExpressionInner,
    pub loc: Span,
}

impl AstNode for Expression {
    type Output = Self;

    fn parse<L: AbsLexer>(parser: &mut Parser<'_, L>) -> RosaRes<Self::Output, Diag> {
        let lhs = parse!(parser => ExpressionInner);

        let mut binary_times: u8 = 0;
        loop {
            lhs = match parser.peek_tok().tt {
                _ => break,
            }
        }
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionInner {
    IntLiteral(u64),
}

impl AstNode for ExpressionInner {
    type Output = Expression;

    fn parse<L: AbsLexer>(parser: &mut Parser<'_, L>) -> RosaRes<Self::Output, Diag> {
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

pub fn parse_intlit_expr(parser: &mut Parser<'_, impl AbsLexer>) -> RosaRes<Expression, Diag> {
    let (i, loc) = expect_token!(parser => [Int(i), i], [FmtToken::IntLiteral]);
    Good(Expression {
        expr: ExpressionInner::IntLiteral(i),
        loc,
    })
}
