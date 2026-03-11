// ============================================================================
// AlgoSpeak Compiler — Recursive Descent Parser
// ============================================================================
// Transforms the flat token stream into an AST (Abstract Syntax Tree).
//
// The parser is pure recursive descent with the following precedence levels
// (lowest to highest):
//   1. or
//   2. and
//   3. comparison  (equals, less than, greater than, …)
//   4. additive    (+, -, minus, plus)
//   5. multiplicative  (*, /, %, divided by, times)
//   6. unary       (-, not)
//   7. primary     (number, string, identifier, array access, function call,
//                   parens, array literal, length of, pop from, dequeue from)
//
// Line-oriented: statements are separated by newlines. The parser silently
// consumes extra newlines between statements.
// ============================================================================

use crate::ast::*;
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // ────────────────────────────────────────────────────────────────────────
    // Public entry point
    // ────────────────────────────────────────────────────────────────────────

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        self.skip_newlines();
        while !self.is_at_end() {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.skip_newlines();
        }
        Ok(Program { statements })
    }

    // ────────────────────────────────────────────────────────────────────────
    // Statement parsing
    // ────────────────────────────────────────────────────────────────────────

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        match self.current_kind() {
            TokenKind::Create => self.parse_var_decl(),
            TokenKind::Set => self.parse_assignment(),
            TokenKind::Show => self.parse_show(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for_each(),
            TokenKind::Algorithm => self.parse_function_def(),
            TokenKind::Reveal => self.parse_return(),
            TokenKind::Stop => {
                self.advance(); // consume 'stop'
                Ok(Stmt::Stop)
            }
            TokenKind::Add => self.parse_natural_add(),
            TokenKind::Subtract => self.parse_natural_subtract(),
            TokenKind::Multiply => self.parse_natural_multiply(),
            TokenKind::Divide => self.parse_natural_divide(),
            TokenKind::Push => self.parse_push(),
            TokenKind::Pop => self.parse_pop_stmt(),
            TokenKind::Enqueue => self.parse_enqueue(),
            TokenKind::Dequeue => self.parse_dequeue_stmt(),
            TokenKind::Sort => self.parse_sort(),
            TokenKind::Reverse => self.parse_reverse(),
            _ => {
                // Try to parse as an expression statement (e.g. a function call)
                let expr = self.parse_expression()?;
                Ok(Stmt::ExprStmt(expr))
            }
        }
    }

    // ── create x as <expr>  |  create stack s  |  create queue q ────────────

    fn parse_var_decl(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Create)?;

        // Check for data structure declarations
        if self.check(TokenKind::Stack) {
            self.advance(); // consume 'stack'
            let name = self.expect_identifier()?;
            return Ok(Stmt::StackDecl { name });
        }

        if self.check(TokenKind::Queue) {
            self.advance(); // consume 'queue'
            let name = self.expect_identifier()?;
            return Ok(Stmt::QueueDecl { name });
        }

        if self.check(TokenKind::Graph) {
            self.advance(); // consume 'graph'
            let name = self.expect_identifier()?;
            // Graph is treated as an adjacency-list array internally
            // For simplicity, we store it similar to an empty array
            return Ok(Stmt::VarDecl {
                name,
                value: Expr::ArrayLiteral(vec![]),
            });
        }

        let name = self.expect_identifier()?;
        self.expect(TokenKind::As)?;
        let value = self.parse_expression()?;
        Ok(Stmt::VarDecl { name, value })
    }

    // ── set x to <expr>  |  set arr[i] to <expr> ───────────────────────────

    fn parse_assignment(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Set)?;
        let name = self.expect_identifier()?;

        // Check for array element assignment: set arr[i] to <expr>
        let index = if self.check(TokenKind::LBracket) {
            self.advance(); // consume '['
            let idx = self.parse_expression()?;
            self.expect(TokenKind::RBracket)?;
            Some(Box::new(idx))
        } else {
            None
        };

        self.expect(TokenKind::To)?;
        let value = self.parse_expression()?;
        Ok(Stmt::Assignment { name, index, value })
    }

    // ── show <expr> ─────────────────────────────────────────────────────────

    fn parse_show(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Show)?;
        let expr = self.parse_expression()?;
        Ok(Stmt::Show(expr))
    }

    // ── if … otherwise … end ────────────────────────────────────────────────

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::If)?;
        let condition = self.parse_expression()?;
        self.expect_newline()?;
        self.skip_newlines();

        let mut then_body = Vec::new();
        while !self.check(TokenKind::Otherwise) && !self.check(TokenKind::End) && !self.is_at_end()
        {
            then_body.push(self.parse_statement()?);
            self.skip_newlines();
        }

        let mut else_body = Vec::new();
        if self.check(TokenKind::Otherwise) {
            self.advance();
            self.skip_newlines();
            while !self.check(TokenKind::End) && !self.is_at_end() {
                else_body.push(self.parse_statement()?);
                self.skip_newlines();
            }
        }

        self.expect(TokenKind::End)?;
        Ok(Stmt::If {
            condition,
            then_body,
            else_body,
        })
    }

    // ── while <cond> … end ──────────────────────────────────────────────────

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::While)?;
        let condition = self.parse_expression()?;
        self.expect_newline()?;
        self.skip_newlines();

        let mut body = Vec::new();
        while !self.check(TokenKind::End) && !self.is_at_end() {
            body.push(self.parse_statement()?);
            self.skip_newlines();
        }

        self.expect(TokenKind::End)?;
        Ok(Stmt::While { condition, body })
    }

    // ── for each <var> in <iterable> … end ──────────────────────────────────

    fn parse_for_each(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::For)?;
        self.expect(TokenKind::Each)?;
        let var = self.expect_identifier()?;
        self.expect(TokenKind::In)?;
        let iterable = self.expect_identifier()?;
        self.expect_newline()?;
        self.skip_newlines();

        let mut body = Vec::new();
        while !self.check(TokenKind::End) && !self.is_at_end() {
            body.push(self.parse_statement()?);
            self.skip_newlines();
        }

        self.expect(TokenKind::End)?;
        Ok(Stmt::ForEach {
            var,
            iterable,
            body,
        })
    }

    // ── algorithm name(params) … end ────────────────────────────────────────
    //    OR natural-language form:
    //    algorithm <name> in <p1> for <p2> … end

    fn parse_function_def(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Algorithm)?;
        let name = self.expect_identifier()?;

        let params;

        if self.check(TokenKind::LParen) {
            // Standard form: algorithm name(params)
            self.advance(); // consume '('
            let mut p = Vec::new();
            if !self.check(TokenKind::RParen) {
                p.push(self.expect_identifier()?);
                while self.check(TokenKind::Comma) {
                    self.advance();
                    p.push(self.expect_identifier()?);
                }
            }
            self.expect(TokenKind::RParen)?;
            params = p;
        } else {
            // Natural-language form: algorithm <name> in <p1> for <p2>
            // Collect params from "in X" and "for Y" clauses
            let mut p = Vec::new();
            if self.check(TokenKind::In) {
                self.advance(); // consume 'in'
                p.push(self.expect_identifier()?);
            }
            if self.check(TokenKind::For) {
                self.advance(); // consume 'for'
                p.push(self.expect_identifier()?);
            }
            params = p;
        }

        self.expect_newline()?;
        self.skip_newlines();

        let mut body = Vec::new();
        while !self.check(TokenKind::End) && !self.is_at_end() {
            body.push(self.parse_statement()?);
            self.skip_newlines();
        }

        self.expect(TokenKind::End)?;
        Ok(Stmt::FunctionDef { name, params, body })
    }

    // ── reveal <expr> ───────────────────────────────────────────────────────

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Reveal)?;
        let expr = self.parse_expression()?;
        Ok(Stmt::Return(expr))
    }

    // ── Natural-language arithmetic ─────────────────────────────────────────

    /// `add <expr> to <name>`  →  set name to name + expr
    fn parse_natural_add(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Add)?;
        let value_expr = self.parse_expression()?;
        self.expect(TokenKind::To)?;
        let target = self.expect_identifier()?;
        Ok(Stmt::Assignment {
            name: target.clone(),
            index: None,
            value: Expr::BinaryOp {
                left: Box::new(Expr::Var(target)),
                op: BinOp::Add,
                right: Box::new(value_expr),
            },
        })
    }

    /// `subtract <expr> from <name>`
    fn parse_natural_subtract(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Subtract)?;
        let value_expr = self.parse_expression()?;
        self.expect(TokenKind::From)?;
        let target = self.expect_identifier()?;
        Ok(Stmt::Assignment {
            name: target.clone(),
            index: None,
            value: Expr::BinaryOp {
                left: Box::new(Expr::Var(target)),
                op: BinOp::Sub,
                right: Box::new(value_expr),
            },
        })
    }

    /// `multiply <name> by <expr>`
    fn parse_natural_multiply(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Multiply)?;
        let target = self.expect_identifier()?;
        self.expect(TokenKind::By)?;
        let value_expr = self.parse_expression()?;
        Ok(Stmt::Assignment {
            name: target.clone(),
            index: None,
            value: Expr::BinaryOp {
                left: Box::new(Expr::Var(target)),
                op: BinOp::Mul,
                right: Box::new(value_expr),
            },
        })
    }

    /// `divide <name> by <expr>`
    fn parse_natural_divide(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Divide)?;
        let target = self.expect_identifier()?;
        self.expect(TokenKind::By)?;
        let value_expr = self.parse_expression()?;
        Ok(Stmt::Assignment {
            name: target.clone(),
            index: None,
            value: Expr::BinaryOp {
                left: Box::new(Expr::Var(target)),
                op: BinOp::Div,
                right: Box::new(value_expr),
            },
        })
    }

    // ── Data structure operations ───────────────────────────────────────────

    /// `push <expr> into <name>`
    fn parse_push(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Push)?;
        let value = self.parse_expression()?;
        self.expect(TokenKind::Into)?;
        let target = self.expect_identifier()?;
        Ok(Stmt::PushStmt { value, target })
    }

    /// `pop from <name>` (as statement)
    fn parse_pop_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Pop)?;
        self.expect(TokenKind::From)?;
        let target = self.expect_identifier()?;
        Ok(Stmt::PopStmt { target })
    }

    /// `enqueue <expr> into <name>`
    fn parse_enqueue(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Enqueue)?;
        let value = self.parse_expression()?;
        self.expect(TokenKind::Into)?;
        let target = self.expect_identifier()?;
        Ok(Stmt::EnqueueStmt { value, target })
    }

    /// `dequeue from <name>` (as statement)
    fn parse_dequeue_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Dequeue)?;
        self.expect(TokenKind::From)?;
        let target = self.expect_identifier()?;
        Ok(Stmt::DequeueStmt { target })
    }

    /// `sort <name>`
    fn parse_sort(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Sort)?;
        let array = self.expect_identifier()?;
        Ok(Stmt::SortStmt { array })
    }

    /// `reverse <name>`
    fn parse_reverse(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Reverse)?;
        let array = self.expect_identifier()?;
        Ok(Stmt::ReverseStmt { array })
    }

    // ────────────────────────────────────────────────────────────────────────
    // Expression parsing — precedence climbing
    // ────────────────────────────────────────────────────────────────────────

    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.check(TokenKind::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        while self.check(TokenKind::And) {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let left = self.parse_additive()?;

        // ── "equals" ────────────────────────────────────────────────────
        if self.check(TokenKind::Equals) {
            self.advance();
            let right = self.parse_additive()?;
            return Ok(Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Eq,
                right: Box::new(right),
            });
        }

        // ── "is less than [or equal to]" / "is greater than [or equal to]" ──
        if self.check(TokenKind::Is) {
            self.advance(); // consume 'is'

            if self.check(TokenKind::Less) {
                self.advance(); // 'less'
                self.expect(TokenKind::Than)?;
                // check "or equal to"
                if self.check(TokenKind::Or) {
                    self.advance(); // 'or'
                    self.expect(TokenKind::Equal)?;
                    self.expect(TokenKind::To)?;
                    let right = self.parse_additive()?;
                    return Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: BinOp::Le,
                        right: Box::new(right),
                    });
                }
                let right = self.parse_additive()?;
                return Ok(Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinOp::Lt,
                    right: Box::new(right),
                });
            }

            if self.check(TokenKind::Greater) {
                self.advance(); // 'greater'
                self.expect(TokenKind::Than)?;
                // check "or equal to"
                if self.check(TokenKind::Or) {
                    self.advance(); // 'or'
                    self.expect(TokenKind::Equal)?;
                    self.expect(TokenKind::To)?;
                    let right = self.parse_additive()?;
                    return Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: BinOp::Ge,
                        right: Box::new(right),
                    });
                }
                let right = self.parse_additive()?;
                return Ok(Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinOp::Gt,
                    right: Box::new(right),
                });
            }

            // Bare "is" without less/greater — treat as equality
            // (e.g. "x is 5" means x == 5 as a convenience)
            let right = self.parse_additive()?;
            return Ok(Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Eq,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;

        loop {
            if self.check(TokenKind::Plus) {
                self.advance();
                let right = self.parse_multiplicative()?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinOp::Add,
                    right: Box::new(right),
                };
            } else if self.check(TokenKind::Dash) {
                self.advance();
                let right = self.parse_multiplicative()?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinOp::Sub,
                    right: Box::new(right),
                };
            } else if self.check(TokenKind::Minus) {
                // natural language "minus"
                self.advance();
                let right = self.parse_multiplicative()?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinOp::Sub,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;

        loop {
            if self.check(TokenKind::Star) {
                self.advance();
                let right = self.parse_unary()?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinOp::Mul,
                    right: Box::new(right),
                };
            } else if self.check(TokenKind::Slash) {
                self.advance();
                let right = self.parse_unary()?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinOp::Div,
                    right: Box::new(right),
                };
            } else if self.check(TokenKind::Percent) {
                self.advance();
                let right = self.parse_unary()?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinOp::Mod,
                    right: Box::new(right),
                };
            } else if self.check(TokenKind::Divide) {
                // "divided by" in expression context
                self.advance(); // consume 'divided'
                self.expect(TokenKind::By)?;
                let right = self.parse_unary()?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinOp::Div,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.check(TokenKind::Dash) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryMinus(Box::new(expr)));
        }
        if self.check(TokenKind::Not) {
            self.advance();
            let expr = self.parse_unary()?;
            // "not expr" → (expr == 0)  — simple boolean negation
            return Ok(Expr::BinaryOp {
                left: Box::new(expr),
                op: BinOp::Eq,
                right: Box::new(Expr::NumberLit(0)),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current_kind() {
            TokenKind::Number(n) => {
                let val = n;
                self.advance();
                Ok(Expr::NumberLit(val))
            }

            TokenKind::StringLiteral(s) => {
                let val = s;
                self.advance();
                Ok(Expr::StringLit(val))
            }

            TokenKind::LParen => {
                self.advance(); // consume '('
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }

            TokenKind::LBracket => {
                // Array literal: [1, 2, 3]
                self.advance(); // consume '['
                let mut elements = Vec::new();
                if !self.check(TokenKind::RBracket) {
                    elements.push(self.parse_expression()?);
                    while self.check(TokenKind::Comma) {
                        self.advance();
                        elements.push(self.parse_expression()?);
                    }
                }
                self.expect(TokenKind::RBracket)?;
                Ok(Expr::ArrayLiteral(elements))
            }

            TokenKind::Length => {
                // "length of <name>"
                self.advance(); // consume 'length'
                self.expect(TokenKind::Of)?;
                let name = self.expect_identifier()?;
                Ok(Expr::LengthOf(name))
            }

            TokenKind::Pop => {
                // "pop from <name>" as expression
                self.advance(); // consume 'pop'
                self.expect(TokenKind::From)?;
                let target = self.expect_identifier()?;
                Ok(Expr::PopExpr { target })
            }

            TokenKind::Dequeue => {
                // "dequeue from <name>" as expression
                self.advance(); // consume 'dequeue'
                self.expect(TokenKind::From)?;
                let target = self.expect_identifier()?;
                Ok(Expr::DequeueExpr { target })
            }

            TokenKind::Identifier(_) => {
                let name = self.expect_identifier()?;

                // Array access: name[expr]
                if self.check(TokenKind::LBracket) {
                    self.advance(); // consume '['
                    let index = self.parse_expression()?;
                    self.expect(TokenKind::RBracket)?;
                    return Ok(Expr::ArrayAccess {
                        array: name,
                        index: Box::new(index),
                    });
                }

                // Function call: name(args)
                if self.check(TokenKind::LParen) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if !self.check(TokenKind::RParen) {
                        args.push(self.parse_expression()?);
                        while self.check(TokenKind::Comma) {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    return Ok(Expr::FunctionCall { name, args });
                }

                Ok(Expr::Var(name))
            }

            _ => Err(format!(
                "Unexpected token '{}' at line {} col {}",
                self.current_kind(),
                self.current_line(),
                self.current_col()
            )),
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // Token stream helpers
    // ────────────────────────────────────────────────────────────────────────

    fn current_kind(&self) -> TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    fn current_line(&self) -> usize {
        self.tokens.get(self.pos).map(|t| t.line).unwrap_or(0)
    }

    fn current_col(&self) -> usize {
        self.tokens.get(self.pos).map(|t| t.col).unwrap_or(0)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_kind(), TokenKind::Eof)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        std::mem::discriminant(&self.current_kind()) == std::mem::discriminant(&kind)
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), String> {
        if self.check(kind.clone()) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected '{}' but found '{}' at line {} col {}",
                kind,
                self.current_kind(),
                self.current_line(),
                self.current_col()
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, String> {
        if let TokenKind::Identifier(name) = self.current_kind() {
            self.advance();
            Ok(name)
        } else {
            Err(format!(
                "Expected identifier but found '{}' at line {} col {}",
                self.current_kind(),
                self.current_line(),
                self.current_col()
            ))
        }
    }

    fn expect_newline(&mut self) -> Result<(), String> {
        if self.check(TokenKind::Newline) || self.is_at_end() {
            self.skip_newlines();
            Ok(())
        } else {
            Err(format!(
                "Expected newline but found '{}' at line {} col {}",
                self.current_kind(),
                self.current_line(),
                self.current_col()
            ))
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(TokenKind::Newline) {
            self.advance();
        }
    }
}
