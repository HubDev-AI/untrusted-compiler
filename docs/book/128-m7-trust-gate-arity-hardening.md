# 128 M7 Slice: Trust-Gate Arity Hardening

This chapter documents a focused M7 trust-boundary contract hardening step for validator/sanitizer gates.

## What it is

Added stricter call-shape enforcement for:
- single-input trust gates (for example `validate.*`, `sanitize.*`, `url.*`, `cors.origin`):
  - require exactly one argument.
- `path.under(base, input)`:
  - requires exactly two arguments.

All related violations emit `E4001` with `security` + `schema` tags.

## Why it exists

Gate functions define trusted transitions. Allowing extra arguments creates ambiguous call intent and weakens boundary readability for humans and tooling.

This slice makes gate shapes deterministic while preserving existing type checks.

## How it works internally

In semantic trust-gate enforcement:
1. single-input gate branch now checks `args.len() == 1` before type checks,
2. path gate branch now checks `args.len() == 2`,
3. existing argument type checks remain in place and now carry security/schema tags consistently.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures for extra-argument gate misuse:
    - `validate.email(input, input)`
    - `path.under(base, input, input)`
  - diagnostic-tag coverage in `compiler/ailang-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of malformed gate arity,
  - deterministic gate-shape diagnostics for editor/tooling consumers.
- Constraint:
  - this slice targets arity/type contracts only; runtime validator behavior is unchanged.

## Failure modes and diagnostics

Examples:
- `validate.email(input, input)` ->
  - `E4001`: trust gate expects exactly one input argument.
- `path.under(base, input, input)` ->
  - `E4001`: path gate expects exactly two arguments.

## Example usage

```ailang
fn validateInput(base: PathSafe, input: Untrusted<String>) -> Int {
  validate.email(input);
  path.under(base, input);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: previously tolerated extra-arg calls now fail fast, requiring cleanup in permissive fixtures.
- Next:
  - continue contract hardening on remaining bridge APIs,
  - align future LSP quick-fix actions with these canonical gate call shapes.
