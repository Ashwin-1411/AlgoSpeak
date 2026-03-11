// ============================================================================
// AlgoSpeak Compiler — Lexer
// ============================================================================
// Character-by-character scanner that produces a Vec<Token>.
//
// Design decisions:
// • Newlines are emitted as tokens so the parser can use them as statement
//   terminators — this keeps the grammar line-oriented like Python.
// • Keywords are detected by a simple match on the lowercased identifier.
// • The lexer is intentionally allocation-light: we only allocate for
//   identifier and string literal values.
// ============================================================================

use crate::token::{Token, TokenKind};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    /// Tokenise the entire source, returning all tokens including a trailing Eof.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            self.skip_spaces_and_tabs();
            self.skip_comment();

            if self.is_eof() {
                tokens.push(Token::new(TokenKind::Eof, self.line, self.col));
                break;
            }

            let ch = self.current();

            // ── Newlines ────────────────────────────────────────────────
            if ch == '\n' {
                // Collapse consecutive newlines into one token.
                if !matches!(tokens.last().map(|t| &t.kind), Some(TokenKind::Newline)) {
                    tokens.push(Token::new(TokenKind::Newline, self.line, self.col));
                }
                self.advance();
                continue;
            }

            if ch == '\r' {
                self.advance();
                continue; // skip CR, will catch LF next
            }

            // ── String literals ─────────────────────────────────────────
            if ch == '"' {
                tokens.push(self.read_string()?);
                continue;
            }

            // ── Number literals ─────────────────────────────────────────
            if ch.is_ascii_digit() {
                tokens.push(self.read_number());
                continue;
            }

            // ── Identifiers & keywords ──────────────────────────────────
            if ch.is_alphabetic() || ch == '_' {
                tokens.push(self.read_identifier());
                continue;
            }

            // ── Single-character symbols ────────────────────────────────
            let tok = match ch {
                '+' => TokenKind::Plus,
                '-' => TokenKind::Dash,
                '*' => TokenKind::Star,
                '/' => TokenKind::Slash,
                '%' => TokenKind::Percent,
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '[' => TokenKind::LBracket,
                ']' => TokenKind::RBracket,
                ',' => TokenKind::Comma,
                _ => {
                    return Err(format!(
                        "Unexpected character '{}' at line {} col {}",
                        ch, self.line, self.col
                    ));
                }
            };
            tokens.push(Token::new(tok, self.line, self.col));
            self.advance();
        }

        Ok(tokens)
    }

    // ── Helpers ─────────────────────────────────────────────────────────────

    fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn current(&self) -> char {
        self.source[self.pos]
    }

    fn advance(&mut self) {
        if !self.is_eof() {
            if self.source[self.pos] == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            self.pos += 1;
        }
    }

    fn skip_spaces_and_tabs(&mut self) {
        while !self.is_eof() {
            let ch = self.current();
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip line comments that start with `#` or `//`.
    fn skip_comment(&mut self) {
        if self.is_eof() {
            return;
        }
        if self.current() == '#' {
            while !self.is_eof() && self.current() != '\n' {
                self.advance();
            }
            return;
        }
        if self.current() == '/'
            && self.pos + 1 < self.source.len()
            && self.source[self.pos + 1] == '/'
        {
            while !self.is_eof() && self.current() != '\n' {
                self.advance();
            }
        }
    }

    fn read_number(&mut self) -> Token {
        let start_col = self.col;
        let start_line = self.line;
        let mut num_str = String::new();
        while !self.is_eof() && self.current().is_ascii_digit() {
            num_str.push(self.current());
            self.advance();
        }
        let value: i64 = num_str.parse().unwrap_or(0);
        Token::new(TokenKind::Number(value), start_line, start_col)
    }

    fn read_identifier(&mut self) -> Token {
        let start_col = self.col;
        let start_line = self.line;
        let mut ident = String::new();
        while !self.is_eof() && (self.current().is_alphanumeric() || self.current() == '_') {
            ident.push(self.current());
            self.advance();
        }

        let kind = match ident.to_lowercase().as_str() {
            "create" => TokenKind::Create,
            "set" => TokenKind::Set,
            "as" => TokenKind::As,
            "to" => TokenKind::To,
            "if" => TokenKind::If,
            "otherwise" => TokenKind::Otherwise,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "each" => TokenKind::Each,
            "in" => TokenKind::In,
            "end" => TokenKind::End,
            "show" => TokenKind::Show,
            "reveal" => TokenKind::Reveal,
            "algorithm" => TokenKind::Algorithm,
            "stop" => TokenKind::Stop,
            "add" => TokenKind::Add,
            "subtract" => TokenKind::Subtract,
            "multiply" => TokenKind::Multiply,
            "divide" => TokenKind::Divide,
            "divided" => TokenKind::Divide,    // "divided by" handled in parser
            "by" => TokenKind::By,
            "from" => TokenKind::From,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            "is" => TokenKind::Is,
            "less" => TokenKind::Less,
            "greater" => TokenKind::Greater,
            "than" => TokenKind::Than,
            "equal" => TokenKind::Equal,
            "equals" => TokenKind::Equals,
            "of" => TokenKind::Of,
            "length" => TokenKind::Length,
            "minus" => TokenKind::Minus,
            "plus" => TokenKind::Plus,
            "times" => TokenKind::Star,
            // ── Data structure keywords ─────────────────────────────────
            "push" => TokenKind::Push,
            "pop" => TokenKind::Pop,
            "into" => TokenKind::Into,
            "enqueue" => TokenKind::Enqueue,
            "dequeue" => TokenKind::Dequeue,
            "connect" => TokenKind::Connect,
            "stack" => TokenKind::Stack,
            "queue" => TokenKind::Queue,
            "graph" => TokenKind::Graph,
            "sort" => TokenKind::Sort,
            "reverse" => TokenKind::Reverse,
            _ => TokenKind::Identifier(ident),
        };

        Token::new(kind, start_line, start_col)
    }

    fn read_string(&mut self) -> Result<Token, String> {
        let start_col = self.col;
        let start_line = self.line;
        self.advance(); // skip opening quote
        let mut value = String::new();
        while !self.is_eof() && self.current() != '"' {
            if self.current() == '\n' {
                return Err(format!(
                    "Unterminated string literal at line {} col {}",
                    start_line, start_col
                ));
            }
            value.push(self.current());
            self.advance();
        }
        if self.is_eof() {
            return Err(format!(
                "Unterminated string literal at line {} col {}",
                start_line, start_col
            ));
        }
        self.advance(); // skip closing quote
        Ok(Token::new(
            TokenKind::StringLiteral(value),
            start_line,
            start_col,
        ))
    }
}
