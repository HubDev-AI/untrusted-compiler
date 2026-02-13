use sec4_core::{
    analyze_program_with_interrupt, parse_source, parse_source_with_interrupt, InterruptSignal,
    Severity,
};
use std::path::Path;

struct AlwaysInterrupted;

impl InterruptSignal for AlwaysInterrupted {
    fn is_interrupted(&self) -> bool {
        true
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
