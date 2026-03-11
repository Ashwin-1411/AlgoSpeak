// ============================================================================
// AlgoSpeak Compiler — Intermediate Representation (AlgoIR)
// ============================================================================
// A flat, linear intermediate representation that sits between the AST and the
// assembly code generator. The IR uses a virtual stack machine model:
//
//   LOAD_CONST 5      — push constant 5
//   LOAD_VAR "x"      — push value of variable x
//   ADD                — pop two, push sum
//   STORE_VAR "y"     — pop TOS and store into y
//
// Benefits:
//   1. Decouples the AST shape from code generation
//   2. Enables optimisation passes (constant folding, strength reduction, DCE)
//   3. Makes the code generator a simple linear walk
// ============================================================================

use crate::ast::*;

/// A single IR instruction.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum IRInst {
    // ── Constants & Variables ───────────────────────────────────────────
    LoadConst(i64),
    LoadVar(String),
    StoreVar(String),

    // ── Arithmetic ─────────────────────────────────────────────────────
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Negate,

    // ── Comparison ─────────────────────────────────────────────────────
    CmpEq,
    CmpNe,
    CmpLt,
    CmpGt,
    CmpLe,
    CmpGe,

    // ── Logic ──────────────────────────────────────────────────────────
    LogicAnd,
    LogicOr,

    // ── Control flow ───────────────────────────────────────────────────
    Label(String),
    Jump(String),
    JumpIfZero(String),

    // ── Functions ──────────────────────────────────────────────────────
    /// Call function with N arguments (args already on stack)
    Call(String, usize),
    /// Return from function (TOS is return value)
    Return,
    /// Start of function: name, params
    FuncBegin(String, Vec<String>),
    /// End of function
    FuncEnd,

    // ── I/O ────────────────────────────────────────────────────────────
    PrintInt,
    PrintNewline,
    PrintString(String),

    // ── Arrays ─────────────────────────────────────────────────────────
    /// Allocate array: name, length
    AllocArray(String, usize),
    /// Store TOS into array element. Index is second-on-stack.
    StoreArrayElem(String),
    /// Load array element. Index on TOS.
    LoadArrayElem(String),
    /// Load array length
    LoadArrayLen(String),
    /// Bounds check: checks TOS (index) against array length
    BoundsCheck(String, usize),

    // ── Data structures ────────────────────────────────────────────────
    AllocStack(String),
    StackPush(String),
    StackPop(String),
    AllocQueue(String),
    QueueEnqueue(String),
    QueueDequeue(String),
    SortArray(String),
    ReverseArray(String),

    // ── Program control ────────────────────────────────────────────────
    Halt,
    HaltError,

    // ── No-op (placeholder for removed instructions) ───────────────────
    Nop,
}

/// Lower an entire AST program into a flat list of IR instructions.
pub struct IRLowering {
    instructions: Vec<IRInst>,
    label_counter: usize,
    /// Collected function definitions (emitted before main code)
    functions: Vec<(String, Vec<String>, Vec<Stmt>)>,
    /// Track loop exit labels for `stop` statement
    loop_exit_stack: Vec<String>,
}

impl IRLowering {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            label_counter: 0,
            functions: Vec::new(),
            loop_exit_stack: Vec::new(),
        }
    }

    fn new_label(&mut self, prefix: &str) -> String {
        self.label_counter += 1;
        format!(".L{}_{}", prefix, self.label_counter)
    }

    fn emit(&mut self, inst: IRInst) {
        self.instructions.push(inst);
    }

    /// Lower the full program, returning the IR instruction stream.
    pub fn lower(mut self, program: &Program) -> Vec<IRInst> {
        // Separate function definitions from top-level statements
        let mut top_level = Vec::new();
        for stmt in &program.statements {
            if let Stmt::FunctionDef { name, params, body } = stmt {
                self.functions.push((name.clone(), params.clone(), body.clone()));
            } else {
                top_level.push(stmt.clone());
            }
        }

        // Emit function definitions first
        let funcs = self.functions.clone();
        for (name, params, body) in &funcs {
            self.emit(IRInst::FuncBegin(name.clone(), params.clone()));
            for stmt in body {
                self.lower_stmt(stmt);
            }
            self.emit(IRInst::FuncEnd);
        }

        // Emit top-level code
        for stmt in &top_level {
            self.lower_stmt(stmt);
        }
        self.emit(IRInst::Halt);

        self.instructions
    }

    fn lower_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, value } => {
                match value {
                    Expr::ArrayLiteral(elements) => {
                        let len = elements.len();
                        self.emit(IRInst::AllocArray(name.clone(), len));
                        for (i, elem) in elements.iter().enumerate() {
                            self.lower_expr(elem);
                            self.emit(IRInst::StoreArrayElem(name.clone()));
                            // The codegen will handle indexing internally
                            // We need to track which element we're storing
                            // Use a convention: after AllocArray, sequential StoreArrayElem
                            // stores elements 0, 1, 2, ...
                            let _ = i; // index tracked by codegen
                        }
                    }
                    _ => {
                        self.lower_expr(value);
                        self.emit(IRInst::StoreVar(name.clone()));
                    }
                }
            }

            Stmt::Assignment { name, index, value } => {
                if let Some(idx_expr) = index {
                    // Array element assignment
                    self.lower_expr(idx_expr);   // index on stack
                    self.lower_expr(value);       // value on stack
                    self.emit(IRInst::StoreArrayElem(name.clone()));
                } else {
                    self.lower_expr(value);
                    self.emit(IRInst::StoreVar(name.clone()));
                }
            }

            Stmt::Show(expr) => {
                match expr {
                    Expr::StringLit(s) => {
                        self.emit(IRInst::PrintString(s.clone()));
                        self.emit(IRInst::PrintNewline);
                    }
                    _ => {
                        self.lower_expr(expr);
                        self.emit(IRInst::PrintInt);
                        self.emit(IRInst::PrintNewline);
                    }
                }
            }

            Stmt::If { condition, then_body, else_body } => {
                let else_label = self.new_label("else");
                let end_label = self.new_label("endif");

                self.lower_expr(condition);
                self.emit(IRInst::JumpIfZero(else_label.clone()));

                for s in then_body {
                    self.lower_stmt(s);
                }
                self.emit(IRInst::Jump(end_label.clone()));

                self.emit(IRInst::Label(else_label));
                for s in else_body {
                    self.lower_stmt(s);
                }

                self.emit(IRInst::Label(end_label));
            }

            Stmt::While { condition, body } => {
                let loop_label = self.new_label("while");
                let end_label = self.new_label("endwhile");

                self.loop_exit_stack.push(end_label.clone());

                self.emit(IRInst::Label(loop_label.clone()));
                self.lower_expr(condition);
                self.emit(IRInst::JumpIfZero(end_label.clone()));

                for s in body {
                    self.lower_stmt(s);
                }
                self.emit(IRInst::Jump(loop_label));
                self.emit(IRInst::Label(end_label));

                self.loop_exit_stack.pop();
            }

            Stmt::ForEach { var, iterable, body } => {
                // Desugar: create hidden index, while index < length ...
                let idx_var = format!("__idx_{}", var);
                let loop_label = self.new_label("foreach");
                let end_label = self.new_label("endforeach");

                self.loop_exit_stack.push(end_label.clone());

                // Initialize index to 0
                self.emit(IRInst::LoadConst(0));
                self.emit(IRInst::StoreVar(idx_var.clone()));

                self.emit(IRInst::Label(loop_label.clone()));

                // Check: idx < length of iterable
                self.emit(IRInst::LoadVar(idx_var.clone()));
                self.emit(IRInst::LoadArrayLen(iterable.clone()));
                self.emit(IRInst::CmpLt);
                self.emit(IRInst::JumpIfZero(end_label.clone()));

                // Load current element into var
                self.emit(IRInst::LoadVar(idx_var.clone()));
                self.emit(IRInst::LoadArrayElem(iterable.clone()));
                self.emit(IRInst::StoreVar(var.clone()));

                for s in body {
                    self.lower_stmt(s);
                }

                // Increment index
                self.emit(IRInst::LoadVar(idx_var.clone()));
                self.emit(IRInst::LoadConst(1));
                self.emit(IRInst::Add);
                self.emit(IRInst::StoreVar(idx_var));

                self.emit(IRInst::Jump(loop_label));
                self.emit(IRInst::Label(end_label));

                self.loop_exit_stack.pop();
            }

            Stmt::FunctionDef { .. } => {
                // Already handled at top level
            }

            Stmt::Return(expr) => {
                self.lower_expr(expr);
                self.emit(IRInst::Return);
            }

            Stmt::Stop => {
                if let Some(label) = self.loop_exit_stack.last().cloned() {
                    self.emit(IRInst::Jump(label));
                } else {
                    self.emit(IRInst::Halt);
                }
            }

            Stmt::ExprStmt(expr) => {
                self.lower_expr(expr);
                // Result discarded — pop is implicit in stack machine
            }

            // ── Data structures ────────────────────────────────────────
            Stmt::StackDecl { name } => {
                self.emit(IRInst::AllocStack(name.clone()));
            }

            Stmt::QueueDecl { name } => {
                self.emit(IRInst::AllocQueue(name.clone()));
            }

            Stmt::PushStmt { value, target } => {
                self.lower_expr(value);
                self.emit(IRInst::StackPush(target.clone()));
            }

            Stmt::PopStmt { target } => {
                self.emit(IRInst::StackPop(target.clone()));
            }

            Stmt::EnqueueStmt { value, target } => {
                self.lower_expr(value);
                self.emit(IRInst::QueueEnqueue(target.clone()));
            }

            Stmt::DequeueStmt { target } => {
                self.emit(IRInst::QueueDequeue(target.clone()));
            }

            Stmt::SortStmt { array } => {
                self.emit(IRInst::SortArray(array.clone()));
            }

            Stmt::ReverseStmt { array } => {
                self.emit(IRInst::ReverseArray(array.clone()));
            }
        }
    }

    fn lower_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::NumberLit(n) => {
                self.emit(IRInst::LoadConst(*n));
            }

            Expr::StringLit(s) => {
                // String literals push a reference — for now, handled specially
                // by PrintString in the show path. If we reach here, push 0.
                self.emit(IRInst::PrintString(s.clone()));
            }

            Expr::Var(name) => {
                self.emit(IRInst::LoadVar(name.clone()));
            }

            Expr::BinaryOp { left, op, right } => {
                self.lower_expr(left);
                self.lower_expr(right);
                match op {
                    BinOp::Add => self.emit(IRInst::Add),
                    BinOp::Sub => self.emit(IRInst::Sub),
                    BinOp::Mul => self.emit(IRInst::Mul),
                    BinOp::Div => self.emit(IRInst::Div),
                    BinOp::Mod => self.emit(IRInst::Mod),
                    BinOp::Eq => self.emit(IRInst::CmpEq),
                    BinOp::Neq => self.emit(IRInst::CmpNe),
                    BinOp::Lt => self.emit(IRInst::CmpLt),
                    BinOp::Gt => self.emit(IRInst::CmpGt),
                    BinOp::Le => self.emit(IRInst::CmpLe),
                    BinOp::Ge => self.emit(IRInst::CmpGe),
                    BinOp::And => self.emit(IRInst::LogicAnd),
                    BinOp::Or => self.emit(IRInst::LogicOr),
                }
            }

            Expr::UnaryMinus(inner) => {
                self.lower_expr(inner);
                self.emit(IRInst::Negate);
            }

            Expr::ArrayLiteral(_) => {
                // Array literals as expressions should only appear in VarDecl
                self.emit(IRInst::LoadConst(0));
            }

            Expr::ArrayAccess { array, index } => {
                self.lower_expr(index);
                self.emit(IRInst::LoadArrayElem(array.clone()));
            }

            Expr::LengthOf(name) => {
                self.emit(IRInst::LoadArrayLen(name.clone()));
            }

            Expr::FunctionCall { name, args } => {
                for arg in args {
                    self.lower_expr(arg);
                }
                self.emit(IRInst::Call(name.clone(), args.len()));
            }

            Expr::PopExpr { target } => {
                self.emit(IRInst::StackPop(target.clone()));
            }

            Expr::DequeueExpr { target } => {
                self.emit(IRInst::QueueDequeue(target.clone()));
            }
        }
    }
}
