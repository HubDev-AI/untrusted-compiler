# 262 M11 Slice: Open-Document Parse Cache

This chapter documents adding a lightweight parse-cache layer for open documents in `sec4audit-language-server`.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- `ServerState` now tracks:
  - `documents` (URI -> text),
  - `parsed_programs` (URI -> parsed `Program`).
- Cache lifecycle:
  - `didOpen`/`didChange` refresh cache from current text.
  - `didClose` evicts cache entry.
- Navigation/refactor/completion flows now reuse cache when available:
  - definition/hover/references/implementation/completion/prepareRename/rename.

## Why it exists

M11 requires incremental analysis direction and responsive editor behavior. Reusing parsed ASTs for open documents removes repeated parse work across frequent LSP requests on unchanged source.

## How it works internally

1. On open/change, `refresh_program_cache` attempts parse:
   - parse success -> cache `Program`,
   - parse failure -> remove stale cache entry.
2. Request handlers call `load_cached_program`:
   - use cached `Program` when present,
   - fallback to parse on demand when needed.
3. Cache is URI-keyed and deterministic for open-document state.

## Inputs, outputs, and constraints

- Inputs:
  - LSP document lifecycle notifications and request flows.
- Outputs:
  - same user-facing behavior, with lower repeated parse churn.
- Constraints:
  - cache is parse-level only (no dependency graph or semantic cache yet).

## Failure modes and diagnostics

- invalid source on change evicts cached program for that URI.
- request paths gracefully fallback to parse or empty/null outputs when parsing is unavailable.

## Tests added/updated

`compiler/sec4-lsp` unit tests now also cover:
- parse cache population on valid source,
- parse cache eviction on invalid source update.

## Tradeoffs and next steps

- Tradeoff:
  - this is a local parse cache, not full incremental semantic analysis.
- Next:
  - add dependency-aware invalidation across imports/modules,
  - add semantic cache layers and preemptive deadlines around expensive stages.
