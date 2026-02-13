use sec4_core::{
    analyze_program_with_interrupt, parse_source, parse_source_with_interrupt, InterruptSignal,
    Severity,
};
use std::cell::Cell;
use std::path::Path;

struct AlwaysInterrupted;

impl InterruptSignal for AlwaysInterrupted {
    fn is_interrupted(&self) -> bool {
        true
    }
}

struct InterruptAfterNChecks {
    remaining: Cell<usize>,
}

impl InterruptAfterNChecks {
    fn new(remaining: usize) -> Self {
        Self {
            remaining: Cell::new(remaining),
        }
    }
}

impl InterruptSignal for InterruptAfterNChecks {
    fn is_interrupted(&self) -> bool {
        let remaining = self.remaining.get();
        if remaining == 0 {
            true
        } else {
            self.remaining.set(remaining - 1);
            false
        }
    }
}

#[test]
fn parse_source_with_interrupt_emits_budget_info_diagnostic() {
    let source = "fn main() -> Int {\n  0\n}\n";
    let diagnostics = parse_source_with_interrupt(Path::new("main.ut"), source, &AlwaysInterrupted)
        .expect_err("parse should stop when interrupt is signaled");

    assert!(
        diagnostics.iter().any(|diag| {
            diag.code == "I9001"
                && diag.severity == Severity::Info
                && diag.message.contains("parsing stopped early")
        }),
        "interrupt-aware parse should emit I9001 info diagnostic",
    );
}

#[test]
fn analyze_program_with_interrupt_emits_budget_info_diagnostic() {
    let source = "fn main() -> Int {\n  0\n}\n";
    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let diagnostics = analyze_program_with_interrupt(&program, &AlwaysInterrupted)
        .expect_err("analysis should stop when interrupt is signaled");

    assert!(
        diagnostics.iter().any(|diag| {
            diag.code == "I9001"
                && diag.severity == Severity::Info
                && diag.message.contains("semantic checks stopped early")
        }),
        "interrupt-aware analysis should emit I9001 info diagnostic",
    );
}

#[test]
fn parse_source_with_interrupt_can_stop_during_lexing() {
    let source = format!("// {}\nfn main() -> Int {{\n  0\n}}\n", "a".repeat(10_000));
    let interrupt = InterruptAfterNChecks::new(128);
    let diagnostics = parse_source_with_interrupt(Path::new("main.ut"), &source, &interrupt)
        .expect_err("parse should stop when interrupt is signaled during lexing");

    assert!(
        diagnostics.iter().any(|diag| {
            diag.code == "I9001"
                && diag.severity == Severity::Info
                && diag.message.contains("parsing stopped early")
        }),
        "lexer-stage interruption should emit I9001 info diagnostic",
    );
}
