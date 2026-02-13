# 291 M11 Slice: Workspace Dependency Invalidation for Cached Unopened Files

This chapter documents LSP dependency-graph hardening for unopened workspace files and token-aware import extraction.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`
- `docs/05-sec4-master-roadmap.md`

Key changes:
- `load_cached_program(...)` now caches on-demand parsed workspace files (including unopened files) into:
  - `parsed_programs`
  - `open_document_symbols`
- On-demand parse now also registers dependency edges from source imports, so unopened files participate in invalidation.
- Dependency invalidation now evicts cached unopened dependents when an upstream dependency refreshes or becomes invalid.
- Import extraction moved from line-prefix scanning to token-aware scanning (lexer-based) with a line-based fallback.

## Why it exists

M11 still had a gap: dependency invalidation was effective for open-document import edges, but unopened files loaded during workspace navigation were not first-class cache graph nodes.

That allowed stale parse/symbol cache entries for unopened dependents.

## How it works internally

1. When an uncached document is parsed via `load_cached_program(...)`, the server now:
   - records import edges from the source,
   - stores parsed AST + function symbols in cache maps.
2. `refresh_program_cache(...)` invalidation walks reverse dependency edges and now reaches unopened cached dependents too.
3. Import extraction uses lexer tokens to find `import "..."` literals across multiline forms while skipping comment/string noise by construction.
4. If lexing fails, extraction falls back to conservative line-based parsing.

## Tests added or updated

Updated in `compiler/sec4-lsp/src/main.rs`:
- `refresh_program_cache_invalidates_cached_unopened_dependents`
- `extract_open_document_import_uris_handles_multiline_tokenized_imports`

Existing dependency and import-edge tests continue to pass:
- `refresh_program_cache_invalidates_open_document_dependents`
- `extract_open_document_import_uris_resolves_relative_literals`
- `update_open_document_import_edges_replaces_previous_import_set`

## Tradeoffs and next steps

- Tradeoff:
  - token-aware extraction depends on lexer success; fallback keeps behavior available under lex errors but is less expressive.
- Next:
  - finish remaining M11 item: fully AST-aware code-action rewrites for complex signatures.
