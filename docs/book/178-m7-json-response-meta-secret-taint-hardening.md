# 178 M7 Slice: JSON Response Meta Secret/Taint Hardening

This chapter documents a focused M7 hardening step for `res.okMeta` metadata safety.

## What it is

Extended JSON response checks so `res.okMeta(..., meta)` rejects:
- `Secret<_>` metadata values,
- `Untrusted<_>` metadata values.

Violations emit `E4004` with `security` tags (`secret`/`taint`).

## Why it exists

`res.okMeta` metadata is part of response payloads. Without explicit checks, wrapped secret/untrusted values could leak via success envelopes despite value-argument sink checks.

## How it works internally

In `enforce_json_response_schema_requirements`:
1. preserve existing strict status/schema/value checks,
2. detect `res.okMeta` four-argument form,
3. inspect metadata argument for secret/untrusted wrappers,
4. emit tagged diagnostics when unsafe wrappers are found.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_json_response_meta_secret_argument_type.ai`
    - `invalid_json_response_meta_untrusted_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of secret/untrusted metadata values for `res.okMeta`,
  - deterministic tagged diagnostics for editor/CLI consumers.
- Constraint:
  - this slice targets bridge metadata safety checks only; final runtime envelope behavior remains unchanged.

## Failure modes and diagnostics

Examples:
- `res.okMeta(..., secretMeta)` ->
  - `E4004`: json response meta argument cannot be `Secret<_>`.
- `res.okMeta(..., untrustedMeta)` ->
  - `E4004`: json response meta argument cannot be `Untrusted<_>`.

## Example usage

```ailang
fn ok(schema: String, meta: Int) effects { net } -> Int {
  res.okMeta(201, schema, 1, meta);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: bridge-stage metadata placeholders wrapped as secret/untrusted now fail semantic checks.
- Next:
  - tighten metadata shape typing once a dedicated envelope-meta type is introduced.
