//! Prelude of the lexer used to reduce the lines due to 'use' items and make
//! it cleaner.

// General Lexing tools
pub use crate::abs::{AbsLexer, BufferedLexer};
pub use crate::Lexer;

// Tokens
pub use crate::tokens::{Keyword, Punctuation, Token, TokenType, TokenType::*};

// Other crate preludes
pub(crate) use rosa_comm::prelude::*;
pub(crate) use rosa_errors::prelude::*;
