# 268 M11 Slice: Workspace-Aware Definition and Hover

This chapter documents extending `definition` and `hover` resolution beyond local document symbols to workspace declaration lookup (including unopened files).

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- `textDocument/definition` now resolves declaration via workspace declaration lookup.
- `textDocument/hover` now resolves signature via workspace declaration lookup.
- both flows now support unopened workspace-file declarations discovered by current workspace scan logic.

## Why it exists

Before this slice, definition/hover were effectively local-file for declaration lookup. That produced inconsistent navigation compared to references/rename once symbols were split across files. This aligns core navigation behavior across all major symbol operations.

## How it works internally

1. Parse active document and locate identifier under cursor.
2. Perform workspace declaration search for identifier name (deadline-aware).
3. Definition:
  - return declaration location URI/range from matched symbol.
4. Hover:
  - return matched declaration signature markdown, while keeping hover range on local identifier hit.

## Inputs, outputs, and constraints

- Inputs:
  - active document URI/position and workspace file set.
- Outputs:
  - cross-file definition location and signature hover.
- Constraints:
  - matching is still name-based in this slice.
  - behavior remains bounded by request deadlines.

## Failure modes and diagnostics

- no declaration found in workspace -> `null` definition/hover result.
- deadline expiry can truncate declaration search.

## Tests added/updated

`compiler/sec4-lsp` unit tests now also cover:
- definition resolving to declaration in unopened workspace file.
- hover resolving signature from unopened workspace file declaration.

## Tradeoffs and next steps

- Tradeoff:
  - name-based resolution can be ambiguous in large codebases with duplicate symbols.
- Next:
  - move declaration lookup to stable symbol-ID index and module-aware resolution.
