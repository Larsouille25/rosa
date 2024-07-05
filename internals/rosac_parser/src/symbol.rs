use std::cell::RefCell;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub enum SymbolKind {
    /// Argument of a function
    Arg,
    /// Local variable
    Local,
    /// Global variable
    Global,
}

#[derive(Debug, Clone)]
pub enum SymbolInner {
    /// Undefined symbols are emitted at the parsing stage and replaced by
    /// 'Defined' at name resolution in the Semantincs Analysis stage.
    Undefined(String),
    /// Defined symbols are symbols that have been resolved during the name
    /// resolution in the semantic analyzer.
    Defined {
        name: String,
        kind: SymbolKind,
        // TODO: make `ty` optional so the type of variables can be inferred.
        ty: Type,
        which: u32,
    },
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub s: RefCell<SymbolInner>,
}

impl Symbol {
    pub fn new(name: String) -> Symbol {
        Symbol {
            s: RefCell::new(SymbolInner::Undefined(name)),
        }
    }

    /// Transforms an Undefined symbol to a Defined one, if it is already
    /// defined, does nothing
    pub fn define(&self, kind: SymbolKind, ty: Type, which: u32) {
        let name = match &*self.s.borrow() {
            SymbolInner::Undefined(name) => name.clone(),
            SymbolInner::Defined { .. } => return,
        };

        *self.s.borrow_mut() = SymbolInner::Defined {
            name,
            kind,
            ty,
            which,
        }
    }

    pub fn new_def(name: String, kind: SymbolKind, ty: Type, which: u32) -> Symbol {
        Symbol {
            s: RefCell::new(SymbolInner::Defined {
                name,
                kind,
                ty,
                which,
            }),
        }
    }
}
