use crate::Span;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Keyword {
    Fn,
    Effects,
    Let,
    Const,
    Mut,
    If,
    Else,
    Match,
    Return,
    Struct,
    Enum,
    Resource,
}

impl Keyword {
    pub fn as_str(self) -> &'static str {
        match self {
            Keyword::Fn => "fn",
            Keyword::Effects => "effects",
            Keyword::Let => "let",
            Keyword::Const => "const",
            Keyword::Mut => "mut",
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::Match => "match",
            Keyword::Return => "return",
            Keyword::Struct => "struct",
            Keyword::Enum => "enum",
            Keyword::Resource => "resource",
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Symbol {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    Dot,
    Arrow,
    FatArrow,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    Bang,
    BangEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    AndAnd,
    OrOr,
    Question,
    At,
}

impl Symbol {
    pub fn as_str(self) -> &'static str {
        match self {
            Symbol::LParen => "(",
            Symbol::RParen => ")",
            Symbol::LBrace => "{",
            Symbol::RBrace => "}",
            Symbol::LBracket => "[",
            Symbol::RBracket => "]",
            Symbol::Comma => ",",
            Symbol::Colon => ":",
            Symbol::Semicolon => ";",
            Symbol::Dot => ".",
            Symbol::Arrow => "->",
            Symbol::FatArrow => "=>",
            Symbol::Plus => "+",
            Symbol::Minus => "-",
            Symbol::Star => "*",
            Symbol::Slash => "/",
            Symbol::Percent => "%",
            Symbol::Eq => "=",
            Symbol::EqEq => "==",
            Symbol::Bang => "!",
            Symbol::BangEq => "!=",
            Symbol::Lt => "<",
            Symbol::LtEq => "<=",
            Symbol::Gt => ">",
            Symbol::GtEq => ">=",
            Symbol::AndAnd => "&&",
            Symbol::OrOr => "||",
            Symbol::Question => "?",
            Symbol::At => "@",
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TokenKind {
    Identifier(String),
    Number(String),
    String(String),
    Bool(bool),
    Keyword(Keyword),
    Symbol(Symbol),
    Eof,
}

impl TokenKind {
    pub fn describe(&self) -> String {
        match self {
            TokenKind::Identifier(_) => "identifier".to_string(),
            TokenKind::Number(_) => "number literal".to_string(),
            TokenKind::String(_) => "string literal".to_string(),
            TokenKind::Bool(_) => "bool literal".to_string(),
            TokenKind::Keyword(keyword) => format!("keyword `{keyword}`"),
            TokenKind::Symbol(symbol) => format!("`{symbol}`"),
            TokenKind::Eof => "end of file".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
