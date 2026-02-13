# 161 M7 Slice: err.rateLimit Constructor-Argument Hardening

This chapter documents a focused M7 hardening step for typed rate-limit error constructor usage.

## What it is

Added semantic checks for `err.rateLimit`:
- call shape must be `err.rateLimit(code, message, retryAfterMs)`,
- `code` must be `String`,
- `message` must be `String`,
- `retryAfterMs` must be numeric.

Violations emit `E4001` with `security` tags.

## Why it exists

Rate-limit errors include both semantic labels and numeric retry metadata. Placeholder constructor values weaken the contract and make downstream handling less deterministic.

## How it works internally

In semantic call enforcement:
1. detect `err.rateLimit` calls,
2. enforce exact three-argument shape,
3. enforce typed code/message/retryAfterMs arguments,
4. emit deterministic diagnostics with canonical usage guidance.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_rate_limit_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/sec4-core/tests/c_backend.rs`
    - `compiler/sec4-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of malformed rate-limit constructor values,
  - stable diagnostics for tooling and CI.
- Constraint:
  - runtime error ABI behavior remains unchanged.

## Failure modes and diagnostics

Example:
- `err.rateLimit(1, 2, 3)` ->
  - `E4001`: code/message arguments must be `String`.

## Example usage

```ut
fn makeRateLimit() -> Int {
  err.rateLimit("LIMIT.RATE", "rate limited", 3000);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: permissive constructor placeholders from early bridge fixtures now fail semantic checks.
- Next:
  - continue hardening remaining lower-priority `err.*` helper contracts if stricter typed runtime envelopes are introduced.
