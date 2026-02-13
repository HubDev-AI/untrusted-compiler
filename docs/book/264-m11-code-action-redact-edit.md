# 264 M11 Slice: Code-Action Redact Edit

This chapter documents upgrading the code-action baseline with a concrete auto-edit for secret-leak diagnostics.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- `textDocument/codeAction` parsing now includes request document URI.
- quick-fix generation now has access to current open-document source text.
- for secret-leak diagnostic codes (`E1003`, `E1004`, `E1005`), server can emit an actual workspace edit:
  - selected diagnostic range is wrapped as `redact(<selected>)`.

## Why it exists

M11 requires actionable security-first editor UX. Guidance-only actions help, but concrete safe rewrites reduce fix friction and improve repeatability in AI-assisted workflows.

## How it works internally

1. Parse `textDocument.uri` + diagnostics list from code-action request.
2. For secret-leak codes:
  - extract selected text from diagnostic range (single-line range support),
  - generate quick-fix with `edit.changes[uri]` replacing range with `redact(...)`.
3. Keep existing title-only quick-fixes for other mapped code families.

## Inputs, outputs, and constraints

- Inputs:
  - code-action request diagnostics with ranges.
- Outputs:
  - quick-fix actions, optionally with workspace edit payload.
- Constraints:
  - redact auto-edit currently supports single-line diagnostic ranges.
  - requires source text to be present in open-document LSP state.

## Failure modes and diagnostics

- missing/invalid range text extraction -> quick-fix remains guidance-only (no edit attached).
- unopened document source -> no auto-edit synthesis for that request.

## Tests added/updated

`compiler/sec4-lsp` unit tests now also cover:
- `E1003` code-action on open document emits `newText = "redact(token)"` for selected range.

## Tradeoffs and next steps

- Tradeoff:
  - edit synthesis is intentionally conservative to avoid unsafe rewrites in ambiguous/multi-line ranges.
- Next:
  - add auto-edits for `E1002` (validate/sanitize gate scaffolding) and `E2001` (effect declaration insertion),
  - extend edit synthesis to multi-line ranges with AST-aware rewrite points.
