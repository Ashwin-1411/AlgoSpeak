// ============================================================================
// AlgoSpeak Compiler — Abstract Syntax Tree
// ============================================================================
// Defines every AST node the parser can produce.  Extended with data structure
// operations (stack, queue), built-in functions (sort, reverse), and string
// literal support.
// ============================================================================

/// Binary operators shared by symbolic and natural-language syntax.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

/// An expression that evaluates to a value.
#[derive(Debug, Clone)]
pub enum Expr {
    /// Integer literal, e.g. `42`
    NumberLit(i64),

    /// String literal, e.g. `"hello"`
    StringLit(String),

    /// Variable reference, e.g. `x`
    Var(String),

    /// Binary operation, e.g. `x + 1`
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },

    /// Unary minus, e.g. `-1`
    UnaryMinus(Box<Expr>),

    /// Array literal, e.g. `[1, 2, 3]`
    ArrayLiteral(Vec<Expr>),

    /// Array element access, e.g. `arr[i]`
    ArrayAccess {
        array: String,
        index: Box<Expr>,
    },

    /// `length of arr`
    LengthOf(String),

    /// Function / algorithm call, e.g. `sum(a, b)`
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },

    /// `pop from s` used as expression
    PopExpr {
        target: String,
    },

    /// `dequeue from q` used as expression
    DequeueExpr {
        target: String,
    },
}

/// A statement — the basic unit of execution.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// `create x as <expr>`
    VarDecl {
        name: String,
        value: Expr,
    },

    /// `set x to <expr>`  or  array element assignment `set arr[i] to <expr>`
    Assignment {
        name: String,
        index: Option<Box<Expr>>,  // Some for array element assignment
        value: Expr,
    },

    /// `show <expr>`
    Show(Expr),

    /// `if <cond> ... otherwise ... end`
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },

    /// `while <cond> ... end`
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },

    /// `for each <var> in <iterable> ... end`
    ForEach {
        var: String,
        iterable: String,
        body: Vec<Stmt>,
    },

    /// `algorithm name(params) ... end`
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    /// `reveal <expr>`
    Return(Expr),

    /// `stop`
    Stop,

    /// Expression used as a statement (e.g. bare function call)
    ExprStmt(Expr),

    // ── Data structure operations ───────────────────────────────────────

    /// `create stack s`
    StackDecl {
        name: String,
    },

    /// `create queue q`
    QueueDecl {
        name: String,
    },

    /// `push <expr> into <name>`
    PushStmt {
        value: Expr,
        target: String,
    },

    /// `pop from <name>` (as statement, discards value)
    PopStmt {
        target: String,
    },

    /// `enqueue <expr> into <name>`
    EnqueueStmt {
        value: Expr,
        target: String,
    },

    /// `dequeue from <name>` (as statement, discards value)
    DequeueStmt {
        target: String,
    },

    /// `sort <array_name>`
    SortStmt {
        array: String,
    },

    /// `reverse <array_name>`
    ReverseStmt {
        array: String,
    },
}

/// The top-level program: a sequence of statements.
#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
