# 126 M7 Slice: `path.base` Signature Hardening

This chapter documents a focused M7 constructor-contract hardening step for `PathSafe` base-path creation.

## What it is

Added semantic call-shape enforcement for `path.base(...)`:
- exactly one argument,
- argument must be `String`.

Violations emit `E4001` with `security` tag.

## Why it exists

`path.base` is a trust-related constructor used as the root for `path.under(...)`. Allowing permissive placeholder types weakens the safety story for filesystem path handling.

This slice enforces explicit string base-path construction at compile time.

## How it works internally

In semantic intrinsic enforcement:
1. added `enforce_path_base_signature(...)`,
2. recognized canonical call forms:
   - `path_base`,
   - `path.base`,
3. enforced arity and `String` argument type checks with fix notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixtures for invalid `path.base` argument type and arity
  - diagnostic-tag coverage in `compiler/sec4-core/tests/diagnostic_tags.rs`
  - gate intrinsic CLI `c-bin` integration fixture in `compiler/sec4-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of malformed `path.base(...)` calls,
  - integration fixture updated to string path input (`"/tmp/base"`).
- Constraint:
  - this slice validates signature shape only; path normalization and containment remain runtime/gate responsibilities.

## Failure modes and diagnostics

Examples:
- `path.base(1)` -> `E4001` (`path.base argument must be String`).
- `path.base("/tmp", "extra")` -> `E4001` (`path.base expects exactly one argument`).

## Example usage

```ut
fn config() -> Int {
  let base = path.base("/srv/app");
  base;
  0
}
```

## Tradeoffs and next steps

- Tradeoff: existing placeholder-heavy integration fixtures require normalization to string path literals.
- Next:
  - continue tightening helper constructors in the stdlib bridge,
  - keep constructor diagnostics tagged for upcoming LSP quick-fix support.
