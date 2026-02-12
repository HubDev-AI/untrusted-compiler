use serde::Serialize;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Span {
    pub file: PathBuf,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Span {
    pub fn point(file: impl Into<PathBuf>, line: usize, col: usize) -> Self {
        Self {
            file: file.into(),
            start_line: line,
            start_col: col,
            end_line: line,
            end_col: col,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub span: Span,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn error(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            span,
            notes: Vec::new(),
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn render_plain(&self) -> String {
        let mut lines = vec![format!(
            "{}[{}]: {}",
            self.severity, self.code, self.message
        )];

        lines.push(format!(
            "  --> {}:{}:{}",
            self.span.file.display(),
            self.span.start_line,
            self.span.start_col
        ));

        for note in &self.notes {
            lines.push(format!("  note: {}", note));
        }

        lines.join("\n")
    }

    pub fn render_color(&self) -> String {
        // ANSI colors: red error, yellow warning, blue info.
        let color = match self.severity {
            Severity::Error => "\x1b[31m",
            Severity::Warning => "\x1b[33m",
            Severity::Info => "\x1b[34m",
        };
        let reset = "\x1b[0m";

        let mut lines = vec![format!(
            "{}{}{}[{}]: {}",
            color, self.severity, reset, self.code, self.message
        )];

        lines.push(format!(
            "  --> {}:{}:{}",
            self.span.file.display(),
            self.span.start_line,
            self.span.start_col
        ));

        for note in &self.notes {
            lines.push(format!("  note: {}", note));
        }

        lines.join("\n")
    }
}
