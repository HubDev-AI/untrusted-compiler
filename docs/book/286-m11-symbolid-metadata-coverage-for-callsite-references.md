# 286 M11 Slice: SymbolId Metadata Coverage for Callsite References

This chapter documents test hardening for symbol metadata on callsite-only reference responses.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- extended `references_returns_call_sites_without_declaration_when_excluded` to assert:
  - each returned callsite reference carries non-empty `data.symbolId`.

## Why it exists

Symbol metadata propagation was added for definition/references. This test ensures metadata remains present even when declaration locations are excluded from the response.

## How it works internally

1. Request references with `includeDeclaration = false`.
2. Validate expected callsite lines are returned.
3. Validate each location includes stable symbol metadata.

## Inputs, outputs, and constraints

- Inputs:
  - callsite reference request for a uniquely resolved function symbol.
- Outputs:
  - stronger contract coverage for location metadata shape.
- Constraints:
  - still contingent on unique declaration resolution.

## Failure modes and diagnostics

- metadata regressions now fail deterministic unit tests instead of silently degrading editor integrations.

## Tests added/updated

- Updated:
  - `references_returns_call_sites_without_declaration_when_excluded`

## Tradeoffs and next steps

- Tradeoff:
  - no runtime behavior change; this is confidence/contract coverage.
- Next:
  - continue expanding symbol-ID contract coverage as binding precision improves.
