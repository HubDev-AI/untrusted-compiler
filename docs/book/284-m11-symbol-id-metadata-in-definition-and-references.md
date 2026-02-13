# 284 M11 Slice: Symbol-ID Metadata in Definition and References

This chapter documents propagating stable symbol IDs into LSP navigation payloads.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- definition results now include `data.symbolId` when declaration resolution is unique.
- references results now include `data.symbolId` for declaration and callsite locations when symbol is resolved.
- location builder now supports optional symbol metadata injection.

## Why it exists

M11 still tracks symbol-ID precision as a remaining area. Attaching symbol IDs to navigation payloads is a concrete step toward end-to-end symbol-ID-aware tooling behavior.

## How it works internally

1. Resolve declaration as before.
2. Capture declaration `FunctionSymbol.id`.
3. Attach `data: { symbolId: ... }` to emitted locations:
   - definition target,
   - declaration/callsite references.

## Inputs, outputs, and constraints

- Inputs:
  - resolved declaration metadata and callsite spans.
- Outputs:
  - location payloads with stable symbol identifiers for editor/tooling consumers.
- Constraints:
  - callsite-to-symbol binding is still name-based; metadata does not yet prove semantic binding correctness by itself.

## Failure modes and diagnostics

- when declaration resolution is ambiguous or unavailable, symbol metadata is omitted with existing null/empty behavior.

## Tests added/updated

`compiler/sec4-lsp` now verifies:
- definition payload includes non-empty `data.symbolId` on resolved calls.
- references with declaration included share one consistent `symbolId` value.

## Tradeoffs and next steps

- Tradeoff:
  - metadata propagation improves observability and downstream tooling integration, but full semantic symbol-ID binding remains pending.
- Next:
  - enforce symbol-ID-based callsite resolution paths so rename/references precision is no longer name-driven.
