//! Prelude of the parser used to reduce the lines due to 'use' items and make
//! it cleaner.

// General parsing tools
pub use crate::{
    derive_loc, expect_token, expected_tok_msg, parse, AstNode, AstPart, FmtToken, Location, Parser,
};

// Precedence
pub use crate::precedence::{operator_precedence, PrecedenceValue};

// Main AST node of each module
pub use crate::block::Block;
pub use crate::decl::{Declaration, DeclarationInner, Visibility};
pub use crate::expr::{Associativity, Expression, ExpressionInner, Operator};
pub use crate::stmt::{Statement, StatementInner};
pub use crate::symbol::{Symbol, SymbolKind};
pub use crate::types::{Type, TypeInner};

// Other crates preludes
pub(crate) use rosa_comm::prelude::*;
pub(crate) use rosa_errors::prelude::*;
pub(crate) use rosac_lexer::prelude::*;
