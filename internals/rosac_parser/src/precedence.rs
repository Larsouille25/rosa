use std::collections::HashMap;

use crate::expr::{Associativity, Operator};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PRECEDENCE_TABLE: HashMap<Operator, (Associativity, u16)> = {
        use crate::expr::BinaryOp::*;
        use Associativity::*;
        use Operator::*;

        HashMap::from([
            // (Unary(Negation), (RightToLeft, 8)),
            // (Unary(Not), (RightToLeft, 8)),
            //
            (Binary(Mul), (LeftToRight, 7)),
            (Binary(Div), (LeftToRight, 7)),
            (Binary(Rem), (LeftToRight, 7)),
            //
            (Binary(Add), (LeftToRight, 6)),
            (Binary(Sub), (LeftToRight, 6)),
            //
            (Binary(RShift), (LeftToRight, 5)),
            (Binary(LShift), (LeftToRight, 5)),
            //
            (Binary(CompLT), (LeftToRight, 4)),
            (Binary(CompGT), (LeftToRight, 4)),
            (Binary(CompLTE), (LeftToRight, 4)),
            (Binary(CompLTE), (LeftToRight, 4)),
            //
            (Binary(CompEq), (LeftToRight, 3)),
            (Binary(CompNe), (LeftToRight, 3)),
        ])
    };
}

pub fn operator_precedence(key: impl Into<Operator>) -> (Associativity, u16) {
    let op = key.into();
    PRECEDENCE_TABLE.get(&op).cloned().expect(&format!(
        "The operator `{:?}` is not in the precedence table.",
        op
    ))
}
