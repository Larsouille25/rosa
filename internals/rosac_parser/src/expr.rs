use rosa_comm::Span;
use rosa_errors::{Diag, Fuzzy};
use rosac_lexer::{
    abs::AbsLexer,
    tokens::{
        Punctuation, Token,
        TokenType::{self, *},
    },
};

use crate::{
    expect_token, expected_tok_msg, parse,
    precedence::{operator_precedence, PrecedenceValue},
    AstNode, AstPart, FmtToken, Parser,
};

/// An operator, either a binary operator or a unary operator.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operator {
    Binary(BinaryOp),
    Unary(UnaryOp),
}

impl From<BinaryOp> for Operator {
    fn from(value: BinaryOp) -> Self {
        Self::Binary(value)
    }
}

impl From<UnaryOp> for Operator {
    fn from(value: UnaryOp) -> Self {
        Self::Unary(value)
    }
}

/// Binary Operators
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// Unary Operators
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    // LEFT UNARY OPERATOR
    /// -a
    Negation,
    /// !a
    Not,
    //
    // RIGHT UNARY OPERATOR
    // /// a.?
    // Unwrap,
}

impl UnaryOp {
    pub fn from_punct(punct: Punctuation) -> Option<UnaryOp> {
        use Punctuation as Punct;
        use UnaryOp as UOp;
        Some(match punct {
            Punct::Minus => UOp::Negation,
            Punct::Exclamationmark => UOp::Not,
            _ => return None,
        })
    }

    /// Is the unary operator on the left of the operand
    pub fn is_left(&self) -> bool {
        matches!(self, Self::Negation | Self::Not)
    }

    /// Is the unary operator on the right of the operand
    pub fn is_right(&self) -> bool {
        !self.is_left()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Associativity {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub expr: ExpressionInner,
    pub loc: Span,
}

impl AstNode for Expression {
    type Output = Self;

    fn parse<L: AbsLexer>(parser: &mut Parser<'_, L>) -> Fuzzy<Self::Output, Diag> {
        let mut lhs = parse!(parser => ExpressionInner);

        let mut binary_times: u8 = 0;
        loop {
            lhs = match &parser.peek_tok().tt {
                TokenType::Punct(p)
                    if BinaryOp::from_punct(p.clone()).is_some() && binary_times != 1 =>
                {
                    binary_times += 1;
                    parse!(fn; parser => parse_binary_expr, parser.current_precedence, lhs)
                }
                _ => break,
            };
            if binary_times >= 2 {
                binary_times = 0;
            }
        }

        Fuzzy::Ok(lhs)
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionInner {
    BinaryExpr {
        lhs: Box<Expression>,
        op: BinaryOp,
        rhs: Box<Expression>,
    },
    UnaryExpr {
        op: UnaryOp,
        operand: Box<Expression>,
    },

    // primary expression
    IntLiteral(u64),
}

impl AstNode for ExpressionInner {
    type Output = Expression;

    fn parse<L: AbsLexer>(parser: &mut Parser<'_, L>) -> Fuzzy<Self::Output, Diag> {
        match parser.peek_tok() {
            Token { tt: Int(_), .. } => parse_intlit_expr(parser),
            Token {
                tt: Punct(punct), ..
            } if UnaryOp::from_punct(punct.clone()).is_some_and(|op| op.is_left()) => {
                parse_left_unary_expr(parser)
            }
            t => {
                let t = t.clone();
                Fuzzy::Err(
                    parser
                        .dcx()
                        .struct_err(expected_tok_msg(t.tt, [AstPart::Expression]), t.loc),
                )
            }
        }
    }
}

pub fn parse_intlit_expr(parser: &mut Parser<'_, impl AbsLexer>) -> Fuzzy<Expression, Diag> {
    let (i, loc) = expect_token!(parser => [Int(i), *i], [FmtToken::IntLiteral]);
    Fuzzy::Ok(Expression {
        expr: ExpressionInner::IntLiteral(i),
        loc,
    })
}

pub fn parse_binary_expr(
    parser: &mut Parser<'_, impl AbsLexer>,
    min_precedence: PrecedenceValue,
    mut lhs: Expression,
) -> Fuzzy<Expression, Diag> {
    while let TokenType::Punct(punct) = &parser.peek_tok().tt {
        // check if the punctuation is a binary operator
        let op = match BinaryOp::from_punct(punct.clone()) {
            Some(op) => op,
            None => break,
        };

        // get the precedence of the operator
        let (_, op_precede) = operator_precedence(op.clone());

        // check if the binary operator has more precedence than what's
        // required.
        if op_precede < min_precedence {
            break;
        }

        // consume the binary operator.
        parser.consume_tok();

        // parse the right-hand side of the binary expression
        let mut rhs = parse!(parser => ExpressionInner);

        while let TokenType::Punct(lh_punct) = &parser.peek_tok().tt {
            // check if the lookahead punctuation is a binary operator
            let lh_op = match BinaryOp::from_punct(lh_punct.clone()) {
                Some(op) => op,
                None => break,
            };

            // get the precedence of the lookahead operator
            let (lh_assoc, lh_op_precede) = operator_precedence(lh_op);

            // break if the precendence of the lookahead operator is smaller
            // than the current operator's one. if associativity is LeftToRight
            // we also break if the precedences are equal.
            match lh_assoc {
                Associativity::LeftToRight if lh_op_precede <= op_precede => break,
                Associativity::RightToLeft if lh_op_precede < op_precede => break,
                _ => {}
            }
            rhs = parse!(fn; parser => parse_binary_expr, lh_op_precede, rhs);
        }
        let loc = Span::from_ends(lhs.loc.clone(), rhs.loc.clone());

        lhs = Expression {
            expr: ExpressionInner::BinaryExpr {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            },
            loc,
        };
    }

    Fuzzy::Ok(lhs)
}

pub fn parse_left_unary_expr(parser: &mut Parser<'_, impl AbsLexer>) -> Fuzzy<Expression, Diag> {
    let (punct, lhs) =
        expect_token!(parser => [Punct(punct), punct.clone()], [AstPart::UnaryOperator]);

    let op = match UnaryOp::from_punct(punct.clone()) {
        Some(v) if v.is_left() => v,
        _ => {
            return Fuzzy::Err(
                parser
                    .dcx()
                    .struct_err(expected_tok_msg(punct, ["left unary operator"]), lhs),
            )
        }
    };

    parser.current_precedence = operator_precedence(op.clone()).1;
    let operand = Box::new(parse!(parser => Expression));

    Fuzzy::Ok(Expression {
        loc: Span::from_ends(lhs, operand.loc.clone()),
        expr: ExpressionInner::UnaryExpr { op, operand },
    })
}
