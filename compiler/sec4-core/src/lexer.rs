use crate::diagnostics::{Diagnostic, Severity, Span};
use crate::token::{Keyword, Symbol, Token, TokenKind};
use crate::{InterruptSignal, NeverInterrupt};
use std::path::{Path, PathBuf};

pub fn lex(file: &Path, source: &str) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let interrupt = NeverInterrupt;
    lex_with_interrupt(file, source, &interrupt)
}

pub fn lex_with_interrupt(
    file: &Path,
    source: &str,
    interrupt: &dyn InterruptSignal,
) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let mut lexer = Lexer::new(file.to_path_buf(), source, interrupt);
    lexer.lex_all();

    if lexer.diagnostics.is_empty() {
        Ok(lexer.tokens)
    } else {
        Err(lexer.diagnostics)
    }
}

struct Lexer<'a> {
    file: PathBuf,
    chars: Vec<char>,
    index: usize,
    line: usize,
    col: usize,
    last_line: usize,
    last_col: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    interrupt: &'a dyn InterruptSignal,
    interrupted: bool,
}

impl<'a> Lexer<'a> {
    fn new(file: PathBuf, source: &str, interrupt: &'a dyn InterruptSignal) -> Self {
        Self {
            file,
            chars: source.chars().collect(),
            index: 0,
            line: 1,
            col: 1,
            last_line: 1,
            last_col: 1,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
            interrupt,
            interrupted: false,
        }
    }

    fn lex_all(&mut self) {
        if self.interrupt_if_requested() {
            return;
        }

        while let Some(ch) = self.peek() {
            if self.interrupt_if_requested() {
                break;
            }

            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            if ch == '/' && self.peek_next() == Some('/') {
                self.skip_line_comment();
                continue;
            }

            if Self::is_identifier_start(ch) {
                self.lex_identifier_or_keyword();
                continue;
            }

            if ch.is_ascii_digit() {
                self.lex_number();
                continue;
            }

            match ch {
                '"' => self.lex_string(),
                '(' => self.lex_one_symbol(Symbol::LParen),
                ')' => self.lex_one_symbol(Symbol::RParen),
                '{' => self.lex_one_symbol(Symbol::LBrace),
                '}' => self.lex_one_symbol(Symbol::RBrace),
                '[' => self.lex_one_symbol(Symbol::LBracket),
                ']' => self.lex_one_symbol(Symbol::RBracket),
                ',' => self.lex_one_symbol(Symbol::Comma),
                ':' => self.lex_one_symbol(Symbol::Colon),
                ';' => self.lex_one_symbol(Symbol::Semicolon),
                '.' => self.lex_one_symbol(Symbol::Dot),
                '?' => self.lex_one_symbol(Symbol::Question),
                '@' => self.lex_one_symbol(Symbol::At),
                '+' => self.lex_one_symbol(Symbol::Plus),
                '*' => self.lex_one_symbol(Symbol::Star),
                '%' => self.lex_one_symbol(Symbol::Percent),
                '-' => {
                    let start = self.start_pos();
                    self.advance();
                    if self.consume_if('>') {
                        self.push_token(TokenKind::Symbol(Symbol::Arrow), start);
                    } else {
                        self.push_token(TokenKind::Symbol(Symbol::Minus), start);
                    }
                }
                '=' => {
                    let start = self.start_pos();
                    self.advance();
                    if self.consume_if('>') {
                        self.push_token(TokenKind::Symbol(Symbol::FatArrow), start);
                    } else if self.consume_if('=') {
                        self.push_token(TokenKind::Symbol(Symbol::EqEq), start);
                    } else {
                        self.push_token(TokenKind::Symbol(Symbol::Eq), start);
                    }
                }
                '!' => {
                    let start = self.start_pos();
                    self.advance();
                    if self.consume_if('=') {
                        self.push_token(TokenKind::Symbol(Symbol::BangEq), start);
                    } else {
                        self.push_token(TokenKind::Symbol(Symbol::Bang), start);
                    }
                }
                '<' => {
                    let start = self.start_pos();
                    self.advance();
                    if self.consume_if('=') {
                        self.push_token(TokenKind::Symbol(Symbol::LtEq), start);
                    } else {
                        self.push_token(TokenKind::Symbol(Symbol::Lt), start);
                    }
                }
                '>' => {
                    let start = self.start_pos();
                    self.advance();
                    if self.consume_if('=') {
                        self.push_token(TokenKind::Symbol(Symbol::GtEq), start);
                    } else {
                        self.push_token(TokenKind::Symbol(Symbol::Gt), start);
                    }
                }
                '&' => {
                    let start = self.start_pos();
                    self.advance();
                    if self.consume_if('&') {
                        self.push_token(TokenKind::Symbol(Symbol::AndAnd), start);
                    } else {
                        self.report_error(
                            "L1001",
                            "unexpected character",
                            start,
                            "single `&` is not valid; use `&&`",
                        );
                    }
                }
                '|' => {
                    let start = self.start_pos();
                    self.advance();
                    if self.consume_if('|') {
                        self.push_token(TokenKind::Symbol(Symbol::OrOr), start);
                    } else {
                        self.report_error(
                            "L1001",
                            "unexpected character",
                            start,
                            "single `|` is not valid; use `||`",
                        );
                    }
                }
                '/' => self.lex_one_symbol(Symbol::Slash),
                _ => {
                    let start = self.start_pos();
                    self.advance();
                    self.report_error(
                        "L1001",
                        "unexpected character",
                        start,
                        format!("character `{ch}` is not valid here"),
                    );
                }
            }
        }

        let eof_span = Span::point(self.file.clone(), self.line, self.col);
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: eof_span,
        });
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if self.interrupt_if_requested() {
                return;
            }
            self.advance();
            if ch == '\n' {
                break;
            }
        }
    }

    fn lex_identifier_or_keyword(&mut self) {
        let start = self.start_pos();
        let mut text = String::new();

        while let Some(ch) = self.peek() {
            if self.interrupt_if_requested() {
                return;
            }
            if Self::is_identifier_continue(ch) {
                text.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match text.as_str() {
            "fn" => TokenKind::Keyword(Keyword::Fn),
            "effects" => TokenKind::Keyword(Keyword::Effects),
            "let" => TokenKind::Keyword(Keyword::Let),
            "const" => TokenKind::Keyword(Keyword::Const),
            "mut" => TokenKind::Keyword(Keyword::Mut),
            "if" => TokenKind::Keyword(Keyword::If),
            "else" => TokenKind::Keyword(Keyword::Else),
            "match" => TokenKind::Keyword(Keyword::Match),
            "return" => TokenKind::Keyword(Keyword::Return),
            "struct" => TokenKind::Keyword(Keyword::Struct),
            "enum" => TokenKind::Keyword(Keyword::Enum),
            "resource" => TokenKind::Keyword(Keyword::Resource),
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _ => TokenKind::Identifier(text),
        };

        self.push_token(kind, start);
    }

    fn lex_number(&mut self) {
        let start = self.start_pos();
        let mut text = String::new();

        while let Some(ch) = self.peek() {
            if self.interrupt_if_requested() {
                return;
            }
            if ch.is_ascii_digit() {
                text.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('.') && self.peek_next().is_some_and(|ch| ch.is_ascii_digit()) {
            text.push('.');
            self.advance();

            while let Some(ch) = self.peek() {
                if self.interrupt_if_requested() {
                    return;
                }
                if ch.is_ascii_digit() {
                    text.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.push_token(TokenKind::Number(text), start);
    }

    fn lex_string(&mut self) {
        let start = self.start_pos();
        self.advance();

        let mut value = String::new();
        let mut terminated = false;

        while let Some(ch) = self.peek() {
            if self.interrupt_if_requested() {
                return;
            }
            match ch {
                '"' => {
                    self.advance();
                    terminated = true;
                    break;
                }
                '\\' => {
                    self.advance();
                    if self.interrupt_if_requested() {
                        return;
                    }
                    let Some(escaped) = self.peek() else {
                        break;
                    };
                    self.advance();
                    match escaped {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '"' => value.push('"'),
                        '\\' => value.push('\\'),
                        other => {
                            self.report_error(
                                "L1003",
                                "invalid escape sequence",
                                start,
                                format!("unsupported escape `\\{other}`"),
                            );
                        }
                    }
                }
                '\n' => {
                    self.report_error(
                        "L1002",
                        "unterminated string literal",
                        start,
                        "string literals must be closed before end of line",
                    );
                    self.advance();
                    return;
                }
                _ => {
                    value.push(ch);
                    self.advance();
                }
            }
        }

        if !terminated {
            self.report_error(
                "L1002",
                "unterminated string literal",
                start,
                "reached end of file while scanning string",
            );
            return;
        }

        self.push_token(TokenKind::String(value), start);
    }

    fn lex_one_symbol(&mut self, symbol: Symbol) {
        let start = self.start_pos();
        self.advance();
        self.push_token(TokenKind::Symbol(symbol), start);
    }

    fn push_token(&mut self, kind: TokenKind, start: (usize, usize)) {
        let span = Span {
            file: self.file.clone(),
            start_line: start.0,
            start_col: start.1,
            end_line: self.last_line,
            end_col: self.last_col,
        };

        self.tokens.push(Token { kind, span });
    }

    fn report_error(
        &mut self,
        code: &str,
        message: &str,
        start: (usize, usize),
        note: impl Into<String>,
    ) {
        let span = Span {
            file: self.file.clone(),
            start_line: start.0,
            start_col: start.1,
            end_line: self.last_line,
            end_col: self.last_col,
        };

        self.diagnostics
            .push(Diagnostic::error(code, message, span).with_note(note));
    }

    fn start_pos(&self) -> (usize, usize) {
        (self.line, self.col)
    }

    fn consume_if(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;

        self.last_line = self.line;
        self.last_col = self.col;
        self.index += 1;

        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }

        Some(ch)
    }

    fn is_identifier_start(ch: char) -> bool {
        ch == '_' || ch.is_ascii_alphabetic()
    }

    fn is_identifier_continue(ch: char) -> bool {
        ch == '_' || ch.is_ascii_alphanumeric()
    }

    fn interrupt_if_requested(&mut self) -> bool {
        if self.interrupt.is_interrupted() {
            if !self.interrupted {
                self.interrupted = true;
                self.diagnostics.push(self.interruption_diagnostic());
            }
            true
        } else {
            false
        }
    }

    fn interruption_diagnostic(&self) -> Diagnostic {
        Diagnostic {
            severity: Severity::Info,
            code: "I9001".to_string(),
            message: "analysis budget exceeded; parsing stopped early".to_string(),
            span: Span::point(self.file.clone(), self.line, self.col),
            notes: vec!["increase the analysis budget to complete parsing".to_string()],
            tags: vec!["analysis".to_string()],
        }
    }
}
