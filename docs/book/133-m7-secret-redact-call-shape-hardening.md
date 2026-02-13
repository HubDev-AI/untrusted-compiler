# 133 M7 Slice: Secret Redact Call-Shape Hardening

This chapter documents a focused M7 bridge-contract hardening step for `secrets.redact`.

## What it is

Added semantic argument-shape enforcement for `secrets.redact`:
- requires exactly one argument.

Malformed calls now emit `E4001` with `security` + `secret` tags.

## Why it exists

`secrets.redact` is the canonical safe conversion path from secret values into loggable/display-safe output. Allowing malformed arity weakens boundary clarity and leaves avoidable ambiguity in secret-handling flows.

## How it works internally

In semantic trust/sink contract enforcement:
1. added `enforce_secret_redact_call_shape(...)`,
2. targeted canonical `secret_redact`/`secrets.redact` calls,
3. enforced exact arity (`1`),
4. emitted deterministic `E4001` diagnostics with canonical fix note.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_secret_redact_missing_arg.ut`
  - diagnostic-tag coverage in `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of malformed redact calls,
  - tagged diagnostics suitable for future editor quick-fix surfaces.
- Constraint:
  - this slice is call-shape hardening only; no runtime ABI changes were required.

## Failure modes and diagnostics

Example:
- `secrets.redact()` ->
  - `E4001`: secret redact call expects exactly one argument.

## Example usage

```ut
fn logSafe(token: Secret<String>) -> Int {
  let masked = secrets.redact(token);
  masked;
  0
}
```

## Tradeoffs and next steps

- Tradeoff: malformed redact helper calls that previously parsed now fail at semantic stage.
- Next:
  - harden `secrets.reveal` call shapes similarly while keeping policy enforcement dominant.
