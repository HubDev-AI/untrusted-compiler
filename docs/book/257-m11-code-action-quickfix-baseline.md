# 257 M11 Slice: Code-Action Quickfix Baseline

This chapter documents adding baseline `textDocument/codeAction` quick-fix support to `sec4audit-language-server`.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- LSP now advertises `codeActionProvider` with `quickfix` support.
- LSP now handles `textDocument/codeAction`.
- Server maps selected diagnostic codes to security/effects quick-fix actions:
  - `E1002` -> validate/sanitize gate hint
  - `E1003`/`E1004`/`E1005` -> secret redaction hint
  - `E2001` -> missing-effect declaration hint

## Why it exists

M11 aims for security-first editor UX, not just passive diagnostics. Quick-fix actions provide direct guidance for common secure-by-construction failures and reduce correction latency.

## How it works internally

1. Parse `context.diagnostics` from code-action request.
2. For each diagnostic, inspect `code`.
3. Map recognized codes to deterministic quick-fix titles.
4. Emit `quickfix` code actions with linked diagnostics.
5. De-duplicate repeated quick-fix titles per request.

## Inputs, outputs, and constraints

- Inputs:
  - `textDocument/codeAction` request with diagnostics context.
- Outputs:
  - quick-fix action array.
- Constraints:
  - this slice emits guidance actions (title + diagnostic linkage), not auto-edit rewrites yet.
  - only known code families are mapped.

## Failure modes and diagnostics

- invalid code-action payload -> invalid-request (`-32600`).
- unknown diagnostic code -> no action emitted for that entry.

## Tests added/updated

`compiler/sec4-lsp` unit tests now also cover:
- initialize capability advertisement for `quickfix` code actions.
- code-action request for `E1002` yields validate/sanitize quick-fix action.

## Tradeoffs and next steps

- Tradeoff:
  - action list is code-driven and conservative; no AST rewrite edits yet.
- Next:
  - add real text edits for canonical fixes (`validate`, `sanitize`, `redact`, effect declarations),
  - thread diagnostic tags/notes into richer fix selection and ranking.
