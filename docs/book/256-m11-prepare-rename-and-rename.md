# 256 M11 Slice: Prepare-Rename and Rename

This chapter documents adding baseline rename workflow support (`textDocument/prepareRename` + `textDocument/rename`) to `ailang-language-server`.

## What it is

Updated:
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- LSP now advertises `renameProvider.prepareProvider: true`.
- LSP now handles:
  - `textDocument/prepareRename`
  - `textDocument/rename`
- Rename path now emits workspace edits for:
  - function declaration name span (line-local extraction from `fn <name>`),
  - call-site identifier hits in the current document.

## Why it exists

Rename is a core editor operation needed for day-to-day refactoring. Without it, M11 tooling remains diagnostic-only and cannot support safe iterative code changes.

## How it works internally

1. Parse URI/position (and `newName` for rename).
2. Load and parse the document from current LSP state.
3. Resolve symbol at cursor (function symbol from identifier-under-cursor).
4. `prepareRename`:
   - returns identifier range + placeholder name.
5. `rename`:
   - validates `newName` as identifier-safe.
   - collects call-site identifier spans for that symbol.
   - derives declaration name span from the declaration line text.
   - returns a workspace edit map keyed by document URI.

## Inputs, outputs, and constraints

- Inputs:
  - `textDocument/prepareRename`
  - `textDocument/rename` with `newName`.
- Outputs:
  - prepare range/placeholder object or `null`.
  - workspace edit with per-URI text edits.
- Constraints:
  - current rename scope is document-local.
  - symbol resolution is function-name based for this slice.

## Failure modes and diagnostics

- invalid request payload -> invalid-request (`-32600`).
- invalid rename target (non-identifier) -> invalid-request with explicit message.
- parse/symbol miss -> `prepareRename: null`, `rename: empty change set`.

## Tests added/updated

`compiler/ailang-lsp` unit tests now also cover:
- initialize capability advertisement for rename provider.
- prepareRename returns expected call-site range and placeholder.
- rename returns workspace edits covering declaration + call sites.

## Tradeoffs and next steps

- Tradeoff:
  - no cross-file rename graph yet; no semantic symbol IDs across modules.
- Next:
  - add LSP code actions for security diagnostics (`validate/sanitize/redact`),
  - evolve rename to multi-file/module-aware symbol index.
