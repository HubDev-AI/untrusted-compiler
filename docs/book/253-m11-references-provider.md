# 253 M11 Slice: References Provider

This chapter documents adding `textDocument/references` support to `ailang-language-server`.

## What it is

Updated:
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- LSP now advertises `referencesProvider: true`.
- LSP now handles `textDocument/references`.
- Request parsing now supports `context.includeDeclaration`.
- Reference resolution returns function call-site locations and optionally includes declaration location.

## Why it exists

After diagnostics, definition, and hover, references is the next baseline navigation feature needed for practical editor workflows and safe refactoring.

## How it works internally

1. Parse request URI/position plus `includeDeclaration`.
2. Load source from current LSP document state (or disk fallback for `file://` URI).
3. Parse source via `ailang_core::parse_source`.
4. Find identifier under cursor.
5. Resolve matching function symbol by name.
6. Collect all identifier hits with the same name across function bodies.
7. Return LSP location array:
   - call-site identifiers,
   - declaration prepended when requested.

## Inputs, outputs, and constraints

- Inputs:
  - `textDocument/references` request with `textDocument`, `position`, and optional `context.includeDeclaration`.
- Outputs:
  - location array in JSON-RPC response.
- Constraints:
  - current resolution is name-based and document-local.
  - function-symbol references are prioritized in this v0.1 slice.

## Failure modes and diagnostics

- invalid request payload -> invalid-request response (`-32600`).
- parse failure/no identifier/no symbol match -> empty result array.
- unsupported URI -> invalid-request response.

## Tests added/updated

`compiler/ailang-lsp` unit tests now also cover:
- initialize response advertises references provider.
- references query returns call sites when declaration is excluded.
- references query includes declaration when requested.

## Tradeoffs and next steps

- Tradeoff:
  - no cross-file/module reference index yet.
- Next:
  - add `textDocument/implementation` and `textDocument/completion`,
  - then safe `prepareRename`/`rename` using stronger symbol identity.
