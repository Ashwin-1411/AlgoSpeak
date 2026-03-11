// ============================================================================
// AlgoSpeak Compiler — Semantic Analysis
// ============================================================================
// Single-pass AST walk that enforces safety rules before code generation:
//
//  1. Variables must be declared (`create`) before use.
//  2. Duplicate declarations in the same scope are rejected.
//  3. Functions must be defined before they are called.
//  4. Function call arity must match the definition.
//  5. Array accesses are recorded — bounds checks are emitted in codegen.
//  6. Stack/Queue operations validate target type.
//  7. Sort/Reverse validate target is an array.
//
// The analyser maintains a scope stack (Vec of HashMaps) so that variable
// visibility obeys block structure (if/while/for/function bodies).
// ============================================================================

use std::collections::HashMap;
use crate::ast::*;

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Variable,
    Array,
    Stack,
    Queue,
    Function { arity: usize },
}

pub struct SemanticAnalyzer {
    /// Stack of scopes; the last entry is the innermost scope.
    scopes: Vec<HashMap<String, SymbolKind>>,
    errors: Vec<String>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            errors: Vec::new(),
        }
    }

    /// Analyse a full program.  Returns Ok(()) if no errors, or a combined
    /// error string otherwise.
    pub fn analyze(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.statements {
            self.analyze_stmt(stmt);
        }
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.join("\n"))
        }
    }

    // ── Scope helpers ───────────────────────────────────────────────────────

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str, kind: SymbolKind) {
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(name) {
            self.errors.push(format!(
                "Semantic error: '{}' is already declared in this scope",
                name
            ));
        } else {
            scope.insert(name.to_string(), kind);
        }
    }

    fn resolve(&self, name: &str) -> Option<&SymbolKind> {
        for scope in self.scopes.iter().rev() {
            if let Some(kind) = scope.get(name) {
                return Some(kind);
            }
        }
        None
    }

    // ── Statement analysis ──────────────────────────────────────────────────

    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, value } => {
                self.analyze_expr(value);
                let kind = if matches!(value, Expr::ArrayLiteral(_)) {
                    SymbolKind::Array
                } else {
                    SymbolKind::Variable
                };
                self.declare(name, kind);
            }

            Stmt::Assignment { name, index, value } => {
                if self.resolve(name).is_none() {
                    self.errors.push(format!(
                        "Semantic error: '{}' is not declared",
                        name
                    ));
                }
                if let Some(idx) = index {
                    self.analyze_expr(idx);
                }
                self.analyze_expr(value);
            }

            Stmt::Show(expr) => {
                self.analyze_expr(expr);
            }

            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                self.analyze_expr(condition);
                self.push_scope();
                for s in then_body {
                    self.analyze_stmt(s);
                }
                self.pop_scope();
                self.push_scope();
                for s in else_body {
                    self.analyze_stmt(s);
                }
                self.pop_scope();
            }

            Stmt::While { condition, body } => {
                self.analyze_expr(condition);
                self.push_scope();
                for s in body {
                    self.analyze_stmt(s);
                }
                self.pop_scope();
            }

            Stmt::ForEach { var, iterable, body } => {
                if self.resolve(iterable).is_none() {
                    self.errors.push(format!(
                        "Semantic error: '{}' is not declared",
                        iterable
                    ));
                }
                self.push_scope();
                self.declare(var, SymbolKind::Variable);
                for s in body {
                    self.analyze_stmt(s);
                }
                self.pop_scope();
            }

            Stmt::FunctionDef { name, params, body } => {
                self.declare(
                    name,
                    SymbolKind::Function {
                        arity: params.len(),
                    },
                );
                self.push_scope();
                for p in params {
                    self.declare(p, SymbolKind::Variable);
                }
                for s in body {
                    self.analyze_stmt(s);
                }
                self.pop_scope();
            }

            Stmt::Return(expr) => {
                self.analyze_expr(expr);
            }

            Stmt::Stop => {}

            Stmt::ExprStmt(expr) => {
                self.analyze_expr(expr);
            }

            // ── Data structure operations ───────────────────────────────

            Stmt::StackDecl { name } => {
                self.declare(name, SymbolKind::Stack);
            }

            Stmt::QueueDecl { name } => {
                self.declare(name, SymbolKind::Queue);
            }

            Stmt::PushStmt { value, target } => {
                self.analyze_expr(value);
                match self.resolve(target) {
                    Some(SymbolKind::Stack) => {}
                    Some(_) => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not a stack (cannot push)",
                            target
                        ));
                    }
                    None => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not declared",
                            target
                        ));
                    }
                }
            }

            Stmt::PopStmt { target } => {
                match self.resolve(target) {
                    Some(SymbolKind::Stack) => {}
                    Some(_) => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not a stack (cannot pop)",
                            target
                        ));
                    }
                    None => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not declared",
                            target
                        ));
                    }
                }
            }

            Stmt::EnqueueStmt { value, target } => {
                self.analyze_expr(value);
                match self.resolve(target) {
                    Some(SymbolKind::Queue) => {}
                    Some(_) => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not a queue (cannot enqueue)",
                            target
                        ));
                    }
                    None => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not declared",
                            target
                        ));
                    }
                }
            }

            Stmt::DequeueStmt { target } => {
                match self.resolve(target) {
                    Some(SymbolKind::Queue) => {}
                    Some(_) => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not a queue (cannot dequeue)",
                            target
                        ));
                    }
                    None => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not declared",
                            target
                        ));
                    }
                }
            }

            Stmt::SortStmt { array } => {
                match self.resolve(array) {
                    Some(SymbolKind::Array) => {}
                    Some(_) => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not an array (cannot sort)",
                            array
                        ));
                    }
                    None => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not declared",
                            array
                        ));
                    }
                }
            }

            Stmt::ReverseStmt { array } => {
                match self.resolve(array) {
                    Some(SymbolKind::Array) => {}
                    Some(_) => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not an array (cannot reverse)",
                            array
                        ));
                    }
                    None => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not declared",
                            array
                        ));
                    }
                }
            }
        }
    }

    // ── Expression analysis ─────────────────────────────────────────────────

    fn analyze_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::NumberLit(_) => {}
            Expr::StringLit(_) => {}

            Expr::Var(name) => {
                if self.resolve(name).is_none() {
                    self.errors.push(format!(
                        "Semantic error: '{}' is not declared",
                        name
                    ));
                }
            }

            Expr::BinaryOp { left, op: _, right } => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }

            Expr::UnaryMinus(inner) => {
                self.analyze_expr(inner);
            }

            Expr::ArrayLiteral(elements) => {
                for e in elements {
                    self.analyze_expr(e);
                }
            }

            Expr::ArrayAccess { array, index } => {
                if self.resolve(array).is_none() {
                    self.errors.push(format!(
                        "Semantic error: '{}' is not declared",
                        array
                    ));
                }
                self.analyze_expr(index);
            }

            Expr::LengthOf(name) => {
                if self.resolve(name).is_none() {
                    self.errors.push(format!(
                        "Semantic error: '{}' is not declared",
                        name
                    ));
                }
            }

            Expr::FunctionCall { name, args } => {
                match self.resolve(name) {
                    Some(SymbolKind::Function { arity }) => {
                        if args.len() != *arity {
                            self.errors.push(format!(
                                "Semantic error: '{}' expects {} arguments but got {}",
                                name, arity, args.len()
                            ));
                        }
                    }
                    Some(_) => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not a function",
                            name
                        ));
                    }
                    None => {
                        self.errors.push(format!(
                            "Semantic error: function '{}' is not defined",
                            name
                        ));
                    }
                }
                for a in args {
                    self.analyze_expr(a);
                }
            }

            Expr::PopExpr { target } => {
                match self.resolve(target) {
                    Some(SymbolKind::Stack) => {}
                    Some(_) => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not a stack (cannot pop)",
                            target
                        ));
                    }
                    None => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not declared",
                            target
                        ));
                    }
                }
            }

            Expr::DequeueExpr { target } => {
                match self.resolve(target) {
                    Some(SymbolKind::Queue) => {}
                    Some(_) => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not a queue (cannot dequeue)",
                            target
                        ));
                    }
                    None => {
                        self.errors.push(format!(
                            "Semantic error: '{}' is not declared",
                            target
                        ));
                    }
                }
            }
        }
    }
}
