# 278 M11 Slice: Open-Document Symbol Index and Stable IDs

This chapter documents a first indexed-symbol step for LSP precision and caching.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- `FunctionSymbol` now carries a deterministic `id`.
- server state now caches open-document symbol lists:
  - `open_document_symbols: HashMap<uri, Vec<FunctionSymbol>>`
- symbol cache lifecycle is aligned with parse cache lifecycle:
  - populate on valid open/change parse,
  - evict on parse failure and close.
- declaration scans and completion now reuse cached open-document symbols when available.

## Why it exists

M11 required moving toward indexed symbol precision and better incremental behavior. Before this slice, symbol lists were recomputed repeatedly from ASTs, even for open documents already parsed and cached.

## How it works internally

1. Build stable symbol IDs from declaration span + symbol name.
2. During `refresh_program_cache`, derive and store symbol lists for valid open documents.
3. On close/invalid parse, evict symbol cache entry to avoid stale symbol data.
4. In completion and workspace declaration lookup:
   - use symbol cache for open docs,
   - fall back to parse+collect for uncached/unopened docs.

## Inputs, outputs, and constraints

- Inputs:
  - parsed open-document `Program` values.
- Outputs:
  - deterministic symbol metadata with stable IDs and reduced recomputation for open docs.
- Constraints:
  - callsite-to-declaration linking still uses name-based matching in v0.1.

## Failure modes and diagnostics

- if parse fails, symbol cache entry is removed immediately.
- unresolved/ambiguous declaration handling remains unchanged (safe null/empty responses).

## Tests added/updated

`compiler/sec4-lsp` now covers:
- parse cache test extended to include symbol-cache population/eviction.
- deterministic symbol-id test for equivalent source snapshots.

## Tradeoffs and next steps

- Tradeoff:
  - declaration IDs are stable and cached, but references/rename matching is still primarily name-driven.
- Next:
  - add callsite-level symbol binding so references/rename can be keyed directly by symbol ID end-to-end.
