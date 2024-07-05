//! The semantic analyzer crate, it is responsible of analyzing the semantics
//! of the AST.
use std::collections::HashMap;

use crate::prelude::*;

pub mod name;
pub mod prelude;

/// Symbol Table Error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymTabError {
    /// Tried to exit the global scope.
    ExitGlobalScope,
    /// A symbol already exists with this name in a previous scope
    ShadowSymbol,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    // TODO: maybe refactor 'HashMap<String, Symbol>' to its type
    stack: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    /// Creates a new symbol table
    pub fn new() -> SymbolTable {
        SymbolTable {
            stack: vec![HashMap::new()],
        }
    }

    /// Get a ref to the top most scope
    pub fn top_most(&self) -> &HashMap<String, Symbol> {
        // we are sure it will not panic because the Global scope may not be
        // exited.
        self.stack.last().unwrap()
    }

    /// Get a mutable ref to the top most scope
    pub fn top_most_mut(&mut self) -> &mut HashMap<String, Symbol> {
        // we are sure it will not panic because the Global scope may not be
        // exited.
        self.stack.last_mut().unwrap()
    }

    /// Enter a new scope, in fact create a new HashMap
    pub fn scope_enter(&mut self) {
        self.stack.push(HashMap::new());
    }

    /// Exit a the top most scope, in fact dropping the top most HashMap
    pub fn scope_exit(&mut self) -> Result<(), SymTabError> {
        if self.scope_level() == 1 {
            return Err(SymTabError::ExitGlobalScope);
        }
        // we are sure it will not panic because the Global scope may not be
        // exited.
        self.stack.pop().unwrap();
        Ok(())
    }

    /// Get the number of scopes, in fact the count of how many HashMap there
    /// is.
    pub fn scope_level(&self) -> usize {
        self.stack.len()
    }

    /// Adds a symbol to the top most scope, and check if it's not shadowing a
    /// previous symbol. In fact insert into the top most HashMap
    pub fn scope_bind(&mut self, name: String, sym: Symbol) -> Result<(), SymTabError> {
        if self.scope_lookup(&name).is_some() {
            return Err(SymTabError::ShadowSymbol);
        }
        self.top_most_mut().insert(name, sym);
        Ok(())
    }

    /// Search a symbol from the top most scope to the global scope.
    pub fn scope_lookup(&self, name: &str) -> Option<&Symbol> {
        // we reverse the iterator because we search from the top most to the
        // global scope
        for scope in self.stack.iter().rev() {
            if scope.contains_key(name) {
                // it will always return 'Some' here because we checked if there
                // was the key
                return scope.get(name);
            }
        }
        None
    }

    /// Search a symbol in the top most scope.
    pub fn scope_lookup_top(&self, name: &str) -> Option<&Symbol> {
        if self.top_most().contains_key(name) {
            self.top_most().get(name)
        } else {
            None
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        SymbolTable::new()
    }
}

/// Semantic analyzer of Rosa. It takes a mutable reference to the ast, and
/// modifies it in the name resolution stage.
#[derive(Debug, Clone)]
pub struct SemanticAnalyzer<'r> {
    table: SymbolTable,
    ast: &'r Vec<Declaration>,
    dcx: &'r DiagCtxt<'r>,
    /// Counter used to set the 'which' field of decl's Symbols
    decl_counter: u32,
}

impl<'r> SemanticAnalyzer<'r> {
    pub fn new(ast: &'r mut Vec<Declaration>, dcx: &'r DiagCtxt) -> SemanticAnalyzer<'r> {
        SemanticAnalyzer {
            table: Default::default(),
            ast,
            dcx,
            decl_counter: 0,
        }
    }

    #[must_use]
    pub fn analyze(&mut self) -> Vec<Diag> {
        let mut diags = Vec::new();

        diags.extend(self.resolve_names());

        diags
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn symtbl_default_lvl() {
        let tbl = SymbolTable::new();
        assert_eq!(tbl.scope_level(), 1);
    }

    #[test]
    fn symtbl_binds() {
        let mut tbl = SymbolTable::new();
        let bob = "bob".to_string();
        assert!(tbl.scope_lookup(&bob).is_none());

        let sym = Symbol::new(bob.clone());
        sym.define(
            SymbolKind::Global,
            Type {
                ty: TypeInner::Int,
                loc: Span::ZERO,
            },
            0,
        );
        tbl.scope_bind(bob.clone(), sym).unwrap();

        assert!(tbl.scope_lookup(&bob).is_some());

        tbl.scope_enter();
        assert!(tbl.scope_lookup(&bob).is_some());
    }

    #[test]
    fn symtbl_shadow() {
        let mut tbl = SymbolTable::new();
        let bob = "bob".to_string();
        assert!(tbl.scope_lookup(&bob).is_none());

        let sym = Symbol::new(bob.clone());
        sym.define(
            SymbolKind::Global,
            Type {
                ty: TypeInner::Int,
                loc: Span::ZERO,
            },
            0,
        );
        tbl.scope_bind(bob.clone(), sym.clone()).unwrap();

        assert!(tbl.scope_lookup(&bob).is_some());

        tbl.scope_enter();
        assert!(tbl.scope_lookup(&bob).is_some());

        let res = tbl.scope_bind(bob.clone(), sym);
        assert_eq!(res, Err(SymTabError::ShadowSymbol))
    }

    #[test]
    fn symtbl_try_exit_global_scope() {
        let mut tbl = SymbolTable::new();
        assert_eq!(tbl.scope_exit(), Err(SymTabError::ExitGlobalScope));
    }

    #[test]
    fn symtbl_symbol_not_found() {
        let tbl = SymbolTable::new();
        assert!(matches!(tbl.scope_lookup("Hello"), None));
    }
}
