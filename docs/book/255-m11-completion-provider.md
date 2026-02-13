# 255 M11 Slice: Completion Provider

This chapter documents adding baseline `textDocument/completion` support to `sec4audit-language-server`.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- LSP now advertises `completionProvider` (`resolveProvider: false`).
- LSP now handles `textDocument/completion`.
- Completion responses now include:
  - core language keywords (`fn`, `let`, `return`, `if`, `match`)
  - function symbols discovered in the current document (with signature in `detail`).

## Why it exists

M11 requires practical in-editor assistance beyond diagnostics/navigation. Baseline completion provides immediate developer ergonomics while keeping behavior deterministic and compiler-backed.

## How it works internally

1. Parse completion request URI/position.
2. Load document source from in-memory LSP state (or file fallback).
3. Seed completion items with fixed keyword entries.
4. Parse document and collect function symbols.
5. Append function completions (`kind=Function`) while de-duplicating labels.
6. Return completion item array.

## Inputs, outputs, and constraints

- Inputs:
  - `textDocument/completion` request.
- Outputs:
  - completion array with keyword and function items.
- Constraints:
  - no cross-file completion index yet.
  - no snippet/auto-import/edit actions in this slice.

## Failure modes and diagnostics

- invalid completion payload -> invalid-request (`-32600`).
- parse failure -> keyword-only completion fallback (no hard error).
- unsupported URI -> invalid-request.

## Tests added/updated

`compiler/sec4-lsp` unit tests now also cover:
- initialize response includes completion provider capability.
- completion response contains both a known function symbol and keyword entry.

## Tradeoffs and next steps

- Tradeoff:
  - completion quality is intentionally conservative and document-local.
- Next:
  - add rename (`prepareRename`/`rename`) and code actions,
  - enrich completion with security-first snippets and context-aware ranking.
