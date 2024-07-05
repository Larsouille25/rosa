//! Module responsible for implementing name resolution methods on the Semantic
//! Analyzer.
//!
//! # Name Resolution
//!
//! The name resolution is divided in two phase:
//! 1. Walkthrough the 'Declaration's and bind the decl's name to their symbol
//! 2. Then the rest of the AST, with the scopes, normal

use std::cell::RefCell;

use rosac_parser::symbol::SymbolInner;

use crate::prelude::*;

impl<'r> SemanticAnalyzer<'r> {
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

    pub fn resolve_decl(&mut self, decl: &Declaration) -> Vec<Diag> {
        let mut diags = Vec::new();

        if decl.vis != Visibility::Private {
            diags.push(self.dcx.struct_warn(
                "visibility other than Private is not supported, (treated as private)",
                decl.loc.clone(),
            ));
        }

        let res = match decl.decl {
            DeclarationInner::Function { .. } => self.resolve_fun_decl(&decl),
        };
        diags.extend(res);

        self.decl_counter += 1;

        diags
    }

    pub fn resolve_fun_decl(&mut self, decl: &Declaration) -> Vec<Diag> {
        // TODO: Remove this allow when there will be more Declaration inner.
        #[allow(unreachable_patterns)]
        let (name, args, ret, loc) = match &decl.decl {
            DeclarationInner::Function {
                name, args, ret, ..
            } => (name, args, ret, decl.loc.clone()),
            _ => panic!(
                "resolving names for functions declarations but it's not a function declaration"
            ),
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
                        ret: ret.clone().map(|t| Box::new(t)),
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

    pub fn visit_decl(&mut self, decl: &Declaration) -> Vec<Diag> {
        let mut diags = Vec::new();

        let res = match decl.decl {
            DeclarationInner::Function { .. } => self.visit_fun_decl(&decl),
        };
        diags.extend(res);

        diags
    }

    pub fn visit_fun_decl(&mut self, decl: &Declaration) -> Vec<Diag> {
        // TODO: Remove this allow when there will be more Declaration inner.
        #[allow(unreachable_patterns)]
        let (name, args, ret, block, loc) = match &decl.decl {
            DeclarationInner::Function {
                name,
                args,
                ret,
                block,
            } => (name, args, ret, block, decl.loc.clone()),
            _ => panic!(
                "resolving names for functions declarations but it's not a function declaration"
            ),
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

        self.table.scope_exit();
        diags
    }

    pub fn visit_stmt_block(&mut self, block: &Block<Statement>) -> Vec<Diag> {
        let mut diags = Vec::new();

        self.table.scope_enter();
        for stmt in &block.content {
            diags.extend(self.visit_stmt(stmt));
        }
        self.table.scope_exit();

        diags
    }

    pub fn visit_stmt(&mut self, stmt: &Statement) -> Vec<Diag> {
        let mut diags = Vec::new();
        match &stmt.stmt {
            StatementInner::IfStmt {
                predicate,
                body,
                else_branch,
            } => {
                diags.extend(self.visit_expr(&predicate));
                diags.extend(self.visit_stmt_block(&body));
                if let Some(other) = else_branch {
                    diags.extend(self.visit_stmt_block(&other));
                }
            }
            StatementInner::ExprStmt(expr) | StatementInner::ReturnStmt(Some(expr)) => {
                self.visit_expr(&expr);
            }
            StatementInner::ReturnStmt(None) => {}
        }
        diags
    }

    pub fn visit_expr(&mut self, expr: &Expression) -> Vec<Diag> {
        let mut diags = Vec::new();
        match &expr.expr {
            ExpressionInner::SymbolExpr(symbol) => 'out: {
                dbg!(symbol);
                let name = match symbol.s.borrow().clone() {
                    SymbolInner::Undefined(name) => name,
                    _ => continue 'out,
                }
                if let Some(found) = self.table.scope_lookup(&name) {
                    *symbol.s.borrow_mut() = found.s.borrow().clone();
                } else {
                    todo!("NOT FOUND DIAG HERE.");
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
