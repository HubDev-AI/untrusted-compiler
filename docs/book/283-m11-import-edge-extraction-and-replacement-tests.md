# 283 M11 Slice: Import Edge Extraction and Replacement Tests

This chapter documents additional test coverage for open-document import graph behavior.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- added direct tests for import-edge extraction and import-set replacement behavior:
  - relative import literal resolution and normalization,
  - reverse-edge replacement when an open document changes its import set.

## Why it exists

Import-driven invalidation is correctness-critical. These tests lock in URI normalization and edge replacement semantics so future refactors do not silently break dependency invalidation.

## How it works internally

1. Feed source text with mixed import literal forms.
2. Assert extracted import URI set includes normalized relative targets and excludes non-relative entries.
3. Update the same document with a different import set.
4. Assert reverse-edge map drops stale dependencies and registers new ones.

## Inputs, outputs, and constraints

- Inputs:
  - source URI and import-bearing source text.
- Outputs:
  - deterministic edge extraction/replacement checks.
- Constraints:
  - parser-independent text extraction semantics remain intentionally scoped to relative import literals.

## Failure modes and diagnostics

- mismatched normalization or stale reverse edges now fail targeted unit tests instead of surfacing later as stale editor cache behavior.

## Tests added/updated

`compiler/sec4-lsp` now includes:
- `extract_open_document_import_uris_resolves_relative_literals`
- `update_open_document_import_edges_replaces_previous_import_set`

## Tradeoffs and next steps

- Tradeoff:
  - tests harden text-level behavior but do not yet validate parser-backed module graphs.
- Next:
  - extend coverage once parser-backed dependency extraction is introduced.
