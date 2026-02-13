# 279 M11 Slice: Open-Document Import-Graph Invalidation

This chapter documents dependency-aware cache invalidation for open-document semantic caches.

## What it is

Updated:
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- server state now tracks open-document import edges:
  - `open_document_imports`
  - `reverse_open_document_imports`
- refresh flow now:
  - updates import edges for the changed/open document,
  - invalidates dependent open-document parse/symbol caches via reverse graph traversal.
- close flow now removes import edges for closed documents.

## Why it exists

Without dependency-aware invalidation, dependent documents could retain stale parse/symbol caches after a dependency changed. This slice introduces deterministic cache eviction to keep editor semantics coherent.

## How it works internally

1. Extract relative import URIs from source text (`import "..."` / `import '...'`).
2. Maintain forward and reverse import-edge maps for open documents.
3. On refresh of `X`, walk reverse edges from `X` and evict parse/symbol caches for dependents.
4. Keep changed document refresh behavior unchanged (it is parsed and re-cached after dependent invalidation).

## Inputs, outputs, and constraints

- Inputs:
  - open-document source text and URI.
- Outputs:
  - bounded, deterministic dependent cache invalidation.
- Constraints:
  - current import-edge extraction is text-based and focused on relative literals.
  - unopened-file dependency invalidation is not yet covered.

## Failure modes and diagnostics

- malformed/unsupported import forms are ignored for dependency tracking.
- cache eviction is safe-by-default: unresolved dependency mapping cannot create stale dependent cache entries through this path.

## Tests added/updated

`compiler/ailang-lsp` now includes:
- `refresh_program_cache_invalidates_open_document_dependents`:
  - verifies dependency refresh evicts dependent parse and symbol caches.

## Tradeoffs and next steps

- Tradeoff:
  - this is a pragmatic text-level dependency layer for open docs.
- Next:
  - move to parser-backed module graph invalidation and include unopened workspace documents in incremental dependency invalidation.
