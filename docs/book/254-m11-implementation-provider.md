# 254 M11 Slice: Implementation Provider

This chapter documents adding `textDocument/implementation` support to `ailang-language-server`.

## What it is

Updated:
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- LSP now advertises `implementationProvider: true`.
- LSP now handles `textDocument/implementation`.
- Current implementation behavior maps implementation lookup to existing definition resolution and returns a location array.

## Why it exists

The M11 tooling plan requires baseline navigation parity across common editor actions. `implementation` is expected by editors and allows symbol navigation workflows to remain consistent even before richer type-graph semantics are added.

## How it works internally

1. Parse document URI and cursor position from request payload.
2. Reuse `definition_at_position` for symbol resolution.
3. Convert result to implementation response shape:
   - no match -> empty array
   - match -> one-element location array.

## Inputs, outputs, and constraints

- Inputs:
  - `textDocument/implementation` requests.
- Outputs:
  - location array responses.
- Constraints:
  - resolution is currently function-symbol and document-local.
  - multi-implementation polymorphic cases are not modeled in v0.1.

## Failure modes and diagnostics

- invalid request payload -> invalid-request (`-32600`).
- parse/symbol resolution miss -> empty implementation array.
- unsupported URI -> invalid-request.

## Tests added/updated

`compiler/ailang-lsp` unit tests now also cover:
- initialize capability advertisement for implementation provider.
- implementation lookup resolving a call identifier to declaration location.

## Tradeoffs and next steps

- Tradeoff:
  - current implementation lookup is definition-backed, not a full implementation graph.
- Next:
  - add completion provider,
  - add `prepareRename` + `rename`,
  - evolve symbol identity for cross-file/module precision.
