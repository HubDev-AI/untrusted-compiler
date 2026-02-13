# 274 M11 Slice: Multiline Effect Quickfix Support

This chapter documents extending `E2001` quick-fix editing to function signatures that span multiple lines.

## What it is

Updated:
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- signature scanning now spans from the detected `fn` line to the signature/body boundary.
- missing-effect insertion can target `->` on a later line in multiline signatures.
- append-to-effects behavior also works when `effects { ... }` is on a later signature line.

## Why it exists

The previous quick-fix implementation assumed single-line function signatures. Real handlers often wrap parameters and return type across lines, so missing-effect code actions must still produce precise edits in those layouts.

## How it works internally

1. Find nearest enclosing `fn` line above the diagnostic.
2. Determine a signature scan window ending at the first `{` (or last reachable line before the diagnostic).
3. Search that window for existing `effects { ... }`:
   - append missing effect,
   - skip if already present.
4. If no effects block exists:
   - insert ` effects { <effect> }` before `->`, with `{` as fallback anchor.

## Inputs, outputs, and constraints

- Inputs:
  - `E2001` diagnostic and source text for the active file.
- Outputs:
  - workspace edit for multiline signatures, consistent with single-line behavior.
- Constraints:
  - effects-block parsing remains line-oriented (`effects { ... }` must be on one line for append).

## Failure modes and diagnostics

- if no signature anchor is found, quick-fix is omitted.
- if source text is unavailable, action falls back to guidance-only mode.

## Tests added/updated

`compiler/ailang-lsp` now covers:
- insertion of missing effect for multiline signatures without an effects block,
- append of missing effect for multiline signatures with existing effects.

## Tradeoffs and next steps

- Tradeoff:
  - this is robust for common multiline signatures but still text/line based.
- Next:
  - AST-aware rewrite support for complex formatting (multi-line effects blocks, comments, and advanced signature layouts).
