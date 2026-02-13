# 252 M11 Slice: Definition and Hover Navigation

This chapter documents adding compiler-backed symbol navigation (`definition`) and signature hover support to `sec4audit-language-server`.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- LSP now advertises:
  - `definitionProvider`
  - `hoverProvider`
- LSP now handles:
  - `textDocument/definition`
  - `textDocument/hover`
- server parses open document source, finds identifier at cursor, resolves function symbol, and returns:
  - declaration location for definition
  - function signature markdown for hover

## Why it exists

After diagnostics were wired, editors still lacked basic code navigation and type/symbol insight. Definition and hover are the minimum interactive loop needed before adding richer IDE features (references/completion/rename/code actions).

## How it works internally

1. Parse request payload (`textDocument` URI + `position`).
2. Load document source from in-memory LSP state (fallback to disk for file URIs).
3. Parse source with `sec4_core::parse_source`.
4. Traverse AST to:
   - locate identifier under cursor,
   - collect function symbols and signatures.
5. Match identifier name to symbol name and return:
   - `Location` for definition,
   - hover markdown block for signature.

## Inputs, outputs, and constraints

- Inputs:
  - LSP stdio requests for definition/hover.
- Outputs:
  - JSON-RPC responses with definition location or hover content.
- Constraints:
  - current navigation scope is document-level function symbols.
  - URI support remains `file://` only.

## Failure modes and diagnostics

- invalid request payload -> invalid-request response (`-32600`).
- parse failure/no symbol at cursor/no matching symbol -> `null` result for definition/hover.
- unsupported URI -> invalid-request response.

## Tests added/updated

`compiler/sec4-lsp` unit tests now also cover:
- definition request resolving function call to declaration span.
- hover request returning function signature markdown.

## Tradeoffs and next steps

- Tradeoff:
  - symbol resolution is currently name-based and local to one parsed document.
- Next:
  - add `textDocument/references`,
  - then completion and rename with safer symbol identity mapping.
