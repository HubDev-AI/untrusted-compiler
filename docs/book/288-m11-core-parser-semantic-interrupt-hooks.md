# 288 M11 Slice: Core Parser/Semantic Interrupt Hooks

This chapter documents adding interrupt-aware parse and semantic entrypoints in `ailang-core` and wiring LSP budget/deadline flows to use them.

## What it is

Updated:
- `compiler/ailang-core/src/lib.rs`
- `compiler/ailang-core/src/parser.rs`
- `compiler/ailang-core/src/semantic.rs`
- `compiler/ailang-core/tests/interrupts.rs`
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- Introduced shared core interrupt contract:
  - `InterruptSignal`
  - `NeverInterrupt`
- Added interrupt-aware frontend APIs:
  - `parse_source_with_interrupt(...)`
  - `analyze_program_with_interrupt(...)`
  - `analyze_program_with_policy_and_interrupt(...)`
- Parser and semantic passes now check interrupt signals during traversal loops and emit `I9001` info diagnostics when interrupted.
- LSP now uses core interrupt-aware parse/analyze paths for diagnostics and deadline-aware uncached parse loading.

## Why it exists

Before this slice, LSP could only short-circuit between stages. A long parse/semantic traversal could still run past budget once started. Core interrupt hooks make cancellation preemptive inside those stages.

## How it works internally

1. LSP constructs a budget/deadline signal (`RequestDeadline`) implementing `InterruptSignal`.
2. Core parse/semantic entrypoints receive that signal.
3. Parser/semantic loops call interruption checks at traversal boundaries.
4. On interruption:
   - traversal stops early,
   - `I9001` info diagnostic is emitted,
   - LSP surfaces that result without duplicate budget markers.

## Tests added/updated

- `compiler/ailang-core/tests/interrupts.rs`
  - `parse_source_with_interrupt_emits_budget_info_diagnostic`
  - `analyze_program_with_interrupt_emits_budget_info_diagnostic`
- Existing `compiler/ailang-lsp` test suite remains green with budget/deadline scenarios.

## Tradeoffs and next steps

- Tradeoff:
  - cancellation checks are now embedded in parser/semantic hot loops, which adds minimal branch overhead for better responsiveness.
- Next:
  - extend the same interrupt contract to any remaining heavy frontend phases (for example lexer-time checks) for full pipeline uniformity.
