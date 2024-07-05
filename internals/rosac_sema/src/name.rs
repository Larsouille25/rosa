//! Module responsible for implementing name resolution methods on the Semantic
//! Analyzer.
//!
//! # Name Resolution
//!
//! The name resolution is divided in two phase:
//! 1. Walkthrough the 'Declaration's and bind the decl's name to their symbol
//! 2. Then the rest of the AST, with the scopes, normal

use rosac_parser::symbol::SymbolInner;

use crate::prelude::*;

impl<'r> SemanticAnalyzer<'r> {
    #[must_use]
    pub fn resolve_names(&mut self) -> Vec<Diag> {
        let mut diags = Vec::new();

        // 1st we declare every symbol
        for decl in self.ast.iter() {
            diags.extend(self.resolve_decl(decl));
        }

        // then we revisit the AST
        for decl in self.ast.iter() {
            diags.extend(self.visit_decl(decl));
        }

        diags
    }

    #[must_use]
    pub fn resolve_decl(&mut self, decl: &Declaration) -> Vec<Diag> {
        let mut diags = Vec::new();

        if decl.vis != Visibility::Private {
            diags.push(self.dcx.struct_warn(
                "visibility other than Private is not supported, (treated as private)",
                decl.loc.clone(),
            ));
        }

        let res = match decl.decl {
            DeclarationInner::Function { .. } => self.resolve_fun_decl(decl),
        };
        diags.extend(res);

        self.decl_counter += 1;

        diags
    }

    #[must_use]
    pub fn resolve_fun_decl(&mut self, decl: &Declaration) -> Vec<Diag> {
        let (name, args, ret, loc) = match &decl.decl {
            DeclarationInner::Function {
                name, args, ret, ..
            } => (name, args, ret, decl.loc.clone()),
            // _ => panic!(
            //     "resolving names for functions declarations but it's not a function declaration"
            // ),
        };

        let mut diags = Vec::new();

        let res = self.table.scope_bind(
            name.clone(),
            Symbol::new_def(
                name.clone(),
                SymbolKind::Global,
                Type {
                    ty: TypeInner::FnPtr {
                        args: args.iter().map(|a| a.1.clone()).collect(),
                        ret: ret.clone().map(Box::new),
                    },
                    loc: Span::ZERO,
                },
                self.decl_counter,
            ),
        );
        match res {
            Ok(()) => {}
            Err(SymTabError::ShadowSymbol) => diags.push(self.dcx.struct_err(
                format!("the symbol '{name}' is defined multiple times"),
                loc, // TODO: here it would be cool to just point to the
                     //prototype of the function
            )),
            Err(_) => unreachable!(),
        }

        diags
    }

    #[must_use]
    pub fn visit_decl(&mut self, decl: &Declaration) -> Vec<Diag> {
        let mut diags = Vec::new();

        let res = match decl.decl {
            DeclarationInner::Function { .. } => self.visit_fun_decl(decl),
        };
        diags.extend(res);

        diags
    }

    #[must_use]
    pub fn visit_fun_decl(&mut self, decl: &Declaration) -> Vec<Diag> {
        let (args, block, loc) = match &decl.decl {
            DeclarationInner::Function { args, block, .. } => (args, block, decl.loc.clone()),
            // _ => panic!(
            //     "resolving names for functions declarations but it's not a function declaration"
            // ),
        };
        let mut diags = Vec::new();
        self.table.scope_enter();

        for (i, (name, ty)) in args.iter().enumerate() {
            let i = i as u32;
            let res = self.table.scope_bind(
                name.clone(),
                Symbol::new_def(name.clone(), SymbolKind::Arg, ty.clone(), i),
            );
            match res {
                Ok(()) => {}
                Err(SymTabError::ShadowSymbol) => diags.push(self.dcx.struct_err(
                    format!("the symbol '{name}' is defined multiple times"),
                    loc.clone(), // TODO: here it would be cool to just point to the
                                 //prototype of the function
                )),
                Err(_) => unreachable!(),
            }
        }

        diags.extend(self.visit_stmt_block(block));

        // here we unwrap because it would be a terrible error to let the compiler continue
        // after trying to exit the global scope in this context
        self.table.scope_exit().unwrap();
        diags
    }

    #[must_use]
    pub fn visit_stmt_block(&mut self, block: &Block<Statement>) -> Vec<Diag> {
        let mut diags = Vec::new();

        self.table.scope_enter();
        for stmt in &block.content {
            diags.extend(self.visit_stmt(stmt));
        }
        // here we unwrap because it would be a terrible error to let the compiler continue
        // after trying to exit the global scope in this context
        self.table.scope_exit().unwrap();

        diags
    }

    #[must_use]
    pub fn visit_stmt(&mut self, stmt: &Statement) -> Vec<Diag> {
        let mut diags = Vec::new();
        match &stmt.stmt {
            StatementInner::IfStmt {
                predicate,
                body,
                else_branch,
            } => {
                diags.extend(self.visit_expr(predicate));
                diags.extend(self.visit_stmt_block(body));
                if let Some(other) = else_branch {
                    diags.extend(self.visit_stmt_block(other));
                }
            }
            StatementInner::ExprStmt(expr) | StatementInner::ReturnStmt(Some(expr)) => {
                diags.extend(self.visit_expr(expr));
            }
            StatementInner::ReturnStmt(None) => {}
        }
        diags
    }

    #[must_use]
    pub fn visit_expr(&mut self, expr: &Expression) -> Vec<Diag> {
        let mut diags = Vec::new();
        match &expr.expr {
            ExpressionInner::SymbolExpr(symbol) => 'out: {
                let name = match symbol.s.borrow().clone() {
                    SymbolInner::Undefined(name) => name,
                    // if the symbol is already defined we do nothing but idk if it's a good idea
                    _ => break 'out,
                };
                if let Some(found) = self.table.scope_lookup(&name) {
                    *symbol.s.borrow_mut() = found.s.borrow().clone();
                } else {
                    diags.push(self.dcx.struct_err(
                        format!("cannot found value '{}' in this scope", name),
                        expr.loc.clone(),
                    ))
                }
            }
            ExpressionInner::BinaryExpr { lhs, rhs, .. } => {
                diags.extend(self.visit_expr(lhs));
                diags.extend(self.visit_expr(rhs));
            }
            ExpressionInner::UnaryExpr { operand, .. } => {
                diags.extend(self.visit_expr(operand));
            }
            // we don't use the wildcard `_` pattern because it forces us to
            // adjust this code when a new expression is created
            ExpressionInner::IntLiteral(_)
            | ExpressionInner::BoolLiteral(_)
            | ExpressionInner::CharLiteral(_)
            | ExpressionInner::StrLiteral(_) => {}
        }
        diags
    }
}
