# 148 M7 Slice: Secret Redact Value-Argument Type Hardening

This chapter documents a focused M7 hardening step that enforces secret-typed payloads for `secrets.redact`.

## What it is

Added semantic type enforcement for `secrets.redact`:
- arity remains exactly one argument,
- the argument must be `Secret<_>`.

Violations emit `E4001` with `security` + `secret` tags.

## Why it exists

`secrets.redact` is the canonical safe escape hatch for secret values. Without payload typing, non-secret values could flow through this API shape and weaken the strict secret boundary model.

## How it works internally

In semantic call-shape enforcement for `secrets.redact`:
1. keep existing exact-arity validation,
2. on valid arity, inspect argument type,
3. require `contains_secret()` for the redact payload,
4. emit deterministic `E4001` diagnostics when argument is not secret-typed.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_secret_redact_value_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of non-secret redact payloads,
  - security/secret-tagged diagnostics aligned with secret-handling guarantees.
- Constraint:
  - this slice is semantic-layer hardening only; runtime lowering/ABI remains unchanged.

## Failure modes and diagnostics

Example:
- `secrets.redact("token")` ->
  - `E4001`: secret redact argument must be `Secret<_>`.

## Example usage

```ut
fn scrub(secret: Secret<String>) -> String {
  secrets.redact(secret)
}
```

## Tradeoffs and next steps

- Tradeoff: previously accepted non-secret redact payloads now fail semantic analysis.
- Next:
  - continue tightening remaining secret/capability helper payload contracts where call shape is already strict.
