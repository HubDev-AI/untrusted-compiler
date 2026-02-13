# 266 M11 Slice: Code-Action Validate Edit

This chapter documents extending code-action auto-edits to untrusted-flow diagnostics.

## What it is

Updated:
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- `E1002` quick-fix actions can now emit a concrete edit when source/range text is available.
- generated edit wraps selected range as:
  - `validate(<selected>)?`
- existing guidance-only fallback remains when edit synthesis is not possible.

## Why it exists

Security-first editor UX should reduce repetitive manual fixes. Adding validate-wrapper edits for untrusted-flow diagnostics moves quick-fixes from guidance to action for a common failure class.

## How it works internally

1. For `E1002`, attempt to extract selected expression text from diagnostic range.
2. If extraction succeeds, attach workspace edit replacing range with `validate(<selected>)?`.
3. If extraction fails, emit non-edit guidance action.

## Inputs, outputs, and constraints

- Inputs:
  - `E1002` diagnostics with usable single-line range.
- Outputs:
  - quick-fix action with optional edit payload.
- Constraints:
  - edit synthesis is currently range-text based and single-line.
  - selected wrapper is generic (`validate(...)`) and not sink-specific.

## Failure modes and diagnostics

- missing source text or invalid range -> no edit payload attached.
- multi-line/ambiguous ranges currently fallback to guidance-only action.

## Tests added/updated

`compiler/ailang-lsp` unit tests now also cover:
- `E1002` code-action on open document emits `newText = "validate(input)?"` for selected range.

## Tradeoffs and next steps

- Tradeoff:
  - generic validate wrapper is useful but may need sink-aware gate selection (`sanitize.html`, `url.public`, etc.) for best fix quality.
- Next:
  - route quick-fix generation through diagnostic tags/notes to choose canonical gate family,
  - add auto-fix support for missing effect declarations (`E2001`) and multi-line expressions.
