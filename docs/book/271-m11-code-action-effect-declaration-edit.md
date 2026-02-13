# 271 M11 Slice: Code-Action Effect-Declaration Edit

This chapter documents extending code-action auto-fixes to missing-effect diagnostics (`E2001`).

## What it is

Updated:
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- `E2001` quick-fix can now emit a concrete insertion edit:
  - insert ` effects { <effect> }` into function signature line.
- effect name extraction:
  - derived from backtick token in diagnostic message/notes (for example `` `net` ``).

## Why it exists

Missing-effect diagnostics are frequent in effect-audited workflows. A direct insertion edit reduces friction and keeps effect declarations explicit, aligned with AILang security/audit goals.

## How it works internally

1. Parse missing effect token from diagnostic message metadata.
2. Locate nearest enclosing `fn` signature line above diagnostic range.
3. Choose insertion point before `->` (or before `{` fallback).
4. Attach workspace edit with insertion text.

## Inputs, outputs, and constraints

- Inputs:
  - `E2001` diagnostics with identifiable effect token.
- Outputs:
  - quick-fix action with signature insertion edit.
- Constraints:
  - insertion is single-line and signature-text based.
  - no duplicate-effect de-duplication yet.

## Failure modes and diagnostics

- if effect token or insertion point cannot be determined, action falls back to guidance-only.

## Tests added/updated

`compiler/ailang-lsp` unit tests now also cover:
- `E2001` code-action emits insertion edit ` effects { net }`.

## Tradeoffs and next steps

- Tradeoff:
  - text-level insertion is pragmatic but should evolve toward AST-aware signature rewriting.
- Next:
  - detect/prevent duplicate effect insertion,
  - support multi-line/complex function signature layouts.
