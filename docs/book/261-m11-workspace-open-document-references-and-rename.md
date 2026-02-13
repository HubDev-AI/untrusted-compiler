# 261 M11 Slice: Workspace Open-Document References and Rename

This chapter documents extending LSP references/rename from single-document behavior to deterministic workspace-open-document scope.

## What it is

Updated:
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- `textDocument/references` now aggregates identifier hits across all currently opened LSP documents.
- `textDocument/rename` now emits workspace edits across all currently opened LSP documents.
- `textDocument/prepareRename` now supports cross-file symbols when declaration is in another open document.
- Declaration lookup now scans open documents deterministically (sorted by URI).

## Why it exists

M11 exit criteria call out deterministic references/rename behavior on multi-file fixtures. Single-file-only rename/reference quickly breaks real workflows once symbols are split across modules/files.

## How it works internally

1. Resolve target identifier under cursor in the active document.
2. Build deterministic open-document workspace view:
   - clone `state.documents`,
   - sort by URI,
   - ensure active document is included.
3. For each document:
   - parse source,
   - collect identifier hits for the target name.
4. References:
   - return locations from all matching open documents,
   - prepend declaration location when requested.
5. Rename:
   - generate edits for all matching open documents,
   - insert declaration-name edit in declaration document when available.

## Inputs, outputs, and constraints

- Inputs:
  - references/rename requests on open documents.
- Outputs:
  - references location list spanning open docs.
  - rename workspace edit map spanning open docs.
- Constraints:
  - scope is currently open LSP documents, not full on-disk project/module graph.
  - symbol matching remains name-based in this slice.

## Failure modes and diagnostics

- parse failures in individual open documents are skipped (best-effort aggregation).
- if no declaration is found in open docs:
  - references still return identifier matches,
  - rename returns call-site edits only where found.

## Tests added/updated

`compiler/ailang-lsp` unit tests now also cover:
- references request collecting hits from two open documents.
- rename request returning edits for two open documents (declaration + call sites).

## Tradeoffs and next steps

- Tradeoff:
  - open-document scope keeps behavior deterministic and lightweight, but does not yet cover unopened files.
- Next:
  - add project/module graph indexing for full-workspace rename/references.
  - upgrade name-based matching to stronger symbol identity where available.
