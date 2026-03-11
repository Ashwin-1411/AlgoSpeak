// ============================================================================
// AlgoSpeak Compiler — Token Definitions
// ============================================================================
// Defines all token types produced by the lexer. Each token carries its type,
// line number, and column number for precise error reporting.
// ============================================================================

use std::fmt;

/// Every distinct lexeme the AlgoSpeak lexer can produce.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ── Keywords ────────────────────────────────────────────────────────
    Create,
    Set,
    As,
    To,
    If,
    Otherwise,
    While,
    For,
    Each,
    In,
    End,
    Show,
    Reveal,
    Algorithm,
    Stop,

    // ── Natural-language arithmetic verbs ────────────────────────────────
    Add,
    Subtract,
    Multiply,
    Divide,
    By,
    From,

    // ── Comparison / logic helpers ──────────────────────────────────────
    And,
    Or,
    Not,
    Is,
    Less,
    Greater,
    Than,
    Equal,
    Equals,
    Of,
    Length,
    Minus,

    // ── Data structure keywords ─────────────────────────────────────────
    Push,
    Pop,
    Into,
    Enqueue,
    Dequeue,
    Connect,
    Stack,
    Queue,
    Graph,
    Sort,
    Reverse,

    // ── Literals ────────────────────────────────────────────────────────
    Number(i64),
    StringLiteral(String),
    Identifier(String),

    // ── Symbols ─────────────────────────────────────────────────────────
    Plus,      // +
    Dash,      // - (symbol form)
    Star,      // *
    Slash,     // /
    Percent,   // %
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    Comma,     // ,

    // ── Special ─────────────────────────────────────────────────────────
    Newline,
    Eof,
}

/// A token with position information for error messages.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, col: usize) -> Self {
        Self { kind, line, col }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Create => write!(f, "create"),
            TokenKind::Set => write!(f, "set"),
            TokenKind::As => write!(f, "as"),
            TokenKind::To => write!(f, "to"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Otherwise => write!(f, "otherwise"),
            TokenKind::While => write!(f, "while"),
            TokenKind::For => write!(f, "for"),
            TokenKind::Each => write!(f, "each"),
            TokenKind::In => write!(f, "in"),
            TokenKind::End => write!(f, "end"),
            TokenKind::Show => write!(f, "show"),
            TokenKind::Reveal => write!(f, "reveal"),
            TokenKind::Algorithm => write!(f, "algorithm"),
            TokenKind::Stop => write!(f, "stop"),
            TokenKind::Add => write!(f, "add"),
            TokenKind::Subtract => write!(f, "subtract"),
            TokenKind::Multiply => write!(f, "multiply"),
            TokenKind::Divide => write!(f, "divide"),
            TokenKind::By => write!(f, "by"),
            TokenKind::From => write!(f, "from"),
            TokenKind::And => write!(f, "and"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Not => write!(f, "not"),
            TokenKind::Is => write!(f, "is"),
            TokenKind::Less => write!(f, "less"),
            TokenKind::Greater => write!(f, "greater"),
            TokenKind::Than => write!(f, "than"),
            TokenKind::Equal => write!(f, "equal"),
            TokenKind::Equals => write!(f, "equals"),
            TokenKind::Of => write!(f, "of"),
            TokenKind::Length => write!(f, "length"),
            TokenKind::Minus => write!(f, "minus"),
            TokenKind::Push => write!(f, "push"),
            TokenKind::Pop => write!(f, "pop"),
            TokenKind::Into => write!(f, "into"),
            TokenKind::Enqueue => write!(f, "enqueue"),
            TokenKind::Dequeue => write!(f, "dequeue"),
            TokenKind::Connect => write!(f, "connect"),
            TokenKind::Stack => write!(f, "stack"),
            TokenKind::Queue => write!(f, "queue"),
            TokenKind::Graph => write!(f, "graph"),
            TokenKind::Sort => write!(f, "sort"),
            TokenKind::Reverse => write!(f, "reverse"),
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::StringLiteral(s) => write!(f, "\"{}\"", s),
            TokenKind::Identifier(s) => write!(f, "{}", s),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Dash => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Newline => write!(f, "NEWLINE"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}
