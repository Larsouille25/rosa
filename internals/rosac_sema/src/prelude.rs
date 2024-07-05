//! Prelude of the semantic analyzer used to reduce the lines due to 'use' items and make
//! it cleaner.

// General analysis tools
pub use crate::{SemanticAnalyzer, SymTabError, SymbolTable};

// Other crates preludes
pub(crate) use rosa_comm::prelude::*;
pub(crate) use rosa_errors::prelude::*;
pub(crate) use rosac_parser::prelude::*;
