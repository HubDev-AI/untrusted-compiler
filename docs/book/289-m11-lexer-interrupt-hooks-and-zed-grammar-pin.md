# 289 M11 Slice: Lexer Interrupt Hooks and Zed Grammar Pin

This chapter documents adding lexer-stage interrupt checks in `sec4-core` and pinning the Zed grammar revision to an immutable commit SHA.

## What it is

Updated:
- `compiler/sec4-core/src/lexer.rs`
- `compiler/sec4-core/src/parser.rs`
- `compiler/sec4-core/tests/interrupts.rs`
- `zed-extension/extension.toml`
- `zed-extension/README.md`
- `docs/05-sec4-master-roadmap.md`

Key changes:
- Added `lex_with_interrupt(...)` in `sec4-core` and wired parser entrypoints to use it in interrupt-aware parse flows.
- Lexer now checks the shared `InterruptSignal` during tokenization loops and emits `I9001` info diagnostics when budget/deadline interruption occurs mid-lex.
- Added coverage proving parse interruption can happen during lexing (not only parser traversal).
- Pinned `zed-extension` grammar revision (`grammars.untrusted.rev`) to a concrete commit SHA.

## Why it exists

M11 had two remaining hardening gaps:
- interrupt handling was present in parser/semantic phases but not lexer loops,
- Zed grammar pinning had a validation gate but still used a placeholder revision.

This slice closes both gaps to improve editor responsiveness and release determinism.

## How it works internally

1. LSP request deadlines/budgets already map to `InterruptSignal`.
2. `parse_source_with_interrupt(...)` now calls `lex_with_interrupt(...)`.
3. Lexer checks `interrupt.is_interrupted()` inside top-level and nested scanning loops.
4. On interruption, lexer records `I9001` once and stops scanning.
5. Parser receives lexer diagnostics and returns early with the same info signal.
6. Zed extension metadata now references a concrete grammar commit SHA instead of a placeholder token.

## Tests added or updated

- `compiler/sec4-core/tests/interrupts.rs`
  - `parse_source_with_interrupt_can_stop_during_lexing`
- Existing interrupt tests remain green:
  - `parse_source_with_interrupt_emits_budget_info_diagnostic`
  - `analyze_program_with_interrupt_emits_budget_info_diagnostic`

## Tradeoffs and next steps

- Tradeoff:
  - lexer loops now include interruption checks, which adds a small branch cost in exchange for bounded latency.
- Next:
  - finish remaining M11 items:
    - unopened-file/module dependency invalidation completeness,
    - fully symbol-ID-bound callsite navigation/rename,
    - fully AST-aware code-action rewrites.
