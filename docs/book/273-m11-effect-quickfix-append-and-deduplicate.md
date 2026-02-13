# 273 M11 Slice: Effect Quickfix Append and De-duplicate

This chapter documents refining the `E2001` code-action quick-fix so it can update existing effect declarations safely.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- if a function already has `effects { ... }`, the quick-fix now appends a missing effect inside that block.
- duplicate guard:
  - if the effect is already declared, no `E2001` quick-fix action is emitted.
- fallback behavior retained:
  - if no effects block exists, quick-fix still inserts ` effects { <effect> }` before `->`/`{`.

## Why it exists

The previous behavior only handled signatures without an effects block. In common code, functions already declare some effects, so we need additive edits instead of forcing manual rewrite and we must avoid duplicate edits.

## How it works internally

1. Extract effect token from diagnostic text/notes.
2. Locate nearest enclosing `fn` signature line.
3. If `effects { ... }` exists:
   - parse comma-separated effect list,
   - skip action when effect already present,
   - otherwise emit insertion `, <effect>` before closing brace.
4. If effects block does not exist:
   - emit insertion ` effects { <effect> }` at signature insertion point.

## Inputs, outputs, and constraints

- Inputs:
  - `E2001` diagnostic with effect token.
- Outputs:
  - a concrete workspace edit when a missing effect can be applied safely.
- Constraints:
  - still line-oriented signature parsing (single-line assumptions for this slice).

## Failure modes and diagnostics

- if effect token or signature shape cannot be resolved, the action may be omitted (or fallback to guidance-only when source text is unavailable).

## Tests added/updated

`compiler/sec4-lsp` now covers:
- append missing effect into existing effects block,
- omit effect quick-fix when the effect is already declared,
- existing insertion path for signatures without effects remains covered.

## Tradeoffs and next steps

- Tradeoff:
  - text-level patching is fast and simple but less robust for complex multiline signatures.
- Next:
  - move effect insertion to AST-aware signature rewrite for precise formatting and multiline support.
