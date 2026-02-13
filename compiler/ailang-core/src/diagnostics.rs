use serde::Serialize;
use std::fmt;
use std::fs;
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
    pub tags: Vec<String>,
}

impl Diagnostic {
    pub fn error(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            span,
            notes: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        let tag = tag.into();
        if !self.tags.iter().any(|current| current == &tag) {
            self.tags.push(tag);
        }
        self
    }

    pub fn render_plain(&self) -> String {
        let source = load_source(&self.span.file);
        self.render_with_source(None, source.as_deref())
    }

    pub fn render_color(&self) -> String {
        let source = load_source(&self.span.file);
        self.render_with_source(Some(color_prefix_for(&self.severity)), source.as_deref())
    }

    pub fn render_plain_with_source(&self, source: &str) -> String {
        self.render_with_source(None, Some(source))
    }

    pub fn render_color_with_source(&self, source: &str) -> String {
        self.render_with_source(Some(color_prefix_for(&self.severity)), Some(source))
    }

    fn render_with_source(&self, color_prefix: Option<&str>, source: Option<&str>) -> String {
        let reset = "\x1b[0m";

        let heading = if let Some(prefix) = color_prefix {
            format!(
                "{}{}{}[{}]: {}",
                prefix, self.severity, reset, self.code, self.message
            )
        } else {
            format!("{}[{}]: {}", self.severity, self.code, self.message)
        };

        let mut lines = vec![heading];
        lines.push(format!(
            "  --> {}:{}:{}",
            self.span.file.display(),
            self.span.start_line,
            self.span.start_col
        ));

        if !self.tags.is_empty() {
            lines.push(format!("  tags: {}", self.tags.join(", ")));
        }

        if let Some(snippet) = render_source_snippet(&self.span, source) {
            lines.extend(snippet);
        }

        for note in &self.notes {
            lines.push(format!("  note: {}", note));
        }

        lines.join("\n")
    }
}

fn color_prefix_for(severity: &Severity) -> &'static str {
    match severity {
        Severity::Error => "\x1b[31m",
        Severity::Warning => "\x1b[33m",
        Severity::Info => "\x1b[34m",
    }
}

fn load_source(path: &PathBuf) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn render_source_snippet(span: &Span, source: Option<&str>) -> Option<Vec<String>> {
    let source = source?;
    if span.start_line == 0 || span.start_col == 0 {
        return None;
    }

    let line_text = source.lines().nth(span.start_line.saturating_sub(1))?;
    let width = span.start_line.to_string().len();
    let gutter = " ".repeat(width);
    let marker_start = " ".repeat(span.start_col.saturating_sub(1));
    let marker_end = if span.end_line == span.start_line && span.end_col >= span.start_col {
        span.end_col.saturating_sub(span.start_col)
    } else {
        1
    };
    let marker_len = marker_end.max(1);
    let marker = "^".repeat(marker_len);

    Some(vec![
        format!("  {gutter} |"),
        format!("  {:>width$} | {}", span.start_line, line_text, width = width),
        format!("  {gutter} | {marker_start}{marker}"),
    ])
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, Severity, Span};

    #[test]
    fn render_plain_with_source_includes_tags_and_snippet() {
        let diagnostic = Diagnostic::error(
            "E1002",
            "Untrusted data cannot flow into sink SqlQuery.",
            Span {
                file: "src/main.ai".into(),
                start_line: 2,
                start_col: 20,
                end_line: 2,
                end_col: 26,
            },
        )
        .with_tag("security")
        .with_tag("taint")
        .with_note("use validate.email before constructing SQL params");

        let rendered = diagnostic.render_plain_with_source("fn main() {\n  db.exec(userInput)\n}\n");

        assert!(rendered.contains("error[E1002]: Untrusted data cannot flow into sink SqlQuery."));
        assert!(rendered.contains("--> src/main.ai:2:20"));
        assert!(rendered.contains("tags: security, taint"));
        assert!(rendered.contains("2 |   db.exec(userInput)"));
        assert!(rendered.contains("|                    ^^^^^^"));
        assert!(rendered.contains("note: use validate.email before constructing SQL params"));
    }

    #[test]
    fn render_plain_without_source_omits_snippet() {
        let diagnostic = Diagnostic {
            severity: Severity::Warning,
            code: "W1201".to_string(),
            message: "SELECT without LIMIT".to_string(),
            span: Span::point("missing/source/main.ai", 4, 5),
            notes: vec!["add LIMIT or annotate allow".to_string()],
            tags: vec!["security".to_string()],
        };

        let rendered = diagnostic.render_plain();

        assert!(rendered.contains("warning[W1201]: SELECT without LIMIT"));
        assert!(rendered.contains("--> missing/source/main.ai:4:5"));
        assert!(rendered.contains("tags: security"));
        assert!(!rendered.contains("|"));
        assert!(rendered.contains("note: add LIMIT or annotate allow"));
    }

    #[test]
    fn render_color_with_source_keeps_severity_coloring() {
        let diagnostic = Diagnostic::error(
            "E2001",
            "Function uses effect net but does not declare it.",
            Span::point("src/main.ai", 1, 1),
        );
        let rendered = diagnostic.render_color_with_source("fn fetch() {}\n");
        assert!(rendered.contains("\u{1b}[31merror\u{1b}[0m[E2001]"));
        assert!(rendered.contains("1 | fn fetch() {}"));
    }
}
