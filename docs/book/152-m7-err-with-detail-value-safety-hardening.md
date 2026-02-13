# 152 M7 Slice: err.withDetail Value-Safety Hardening

This chapter documents a focused M7 hardening step for safe error-detail construction.

## What it is

Added semantic checks for `err.withDetail`:
- call shape must be `err.withDetail(error, key, value)`,
- `key` must be `String`,
- `value` cannot be `Secret<_>` or `Untrusted<_>`.

Violations emit `E4001` with security tags (`secret`/`taint` where applicable).

## Why it exists

The error model requires safe, non-secret detail payloads. Allowing secret or untrusted values into `err.withDetail` weakens the no-leak guarantee and can propagate unsafe data into error responses/logs.

## How it works internally

In semantic call enforcement:
1. detect `err.withDetail` calls,
2. enforce exact arity,
3. enforce `String` key argument,
4. reject detail values typed as `Secret<_>` or `Untrusted<_>`,
5. emit deterministic diagnostics with guidance to redact/validate first.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_err_with_detail_secret_value.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/ailang-core/tests/c_backend.rs`
    - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of unsafe error-detail payloads,
  - deterministic, tagged diagnostics for tooling.
- Constraint:
  - this slice hardens semantic validation only; runtime error ABI stays unchanged.

## Failure modes and diagnostics

Examples:
- `err.withDetail(err, 1, value)` ->
  - `E4001`: key argument must be `String`.
- `err.withDetail(err, "token", secret)` ->
  - `E4001`: value argument cannot be `Secret<_>`.

## Example usage

```ailang
fn attach(base: Int, value: Int) -> Int {
  err.withDetail(base, "field", value);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: permissive placeholder detail keys/values used in early bridge fixtures now fail semantic checks.
- Next:
  - extend typed validation across remaining `err.*` helpers (for example `withPath`, `withLimit`, `withDependency`) for full standard-error contract coverage.
