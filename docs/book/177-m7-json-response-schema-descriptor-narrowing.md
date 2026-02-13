# 177 M7 Slice: JSON Response Schema-Descriptor Narrowing

This chapter documents a focused M7 hardening step for JSON response sink schema descriptors.

## What it is

Extended strict JSON response checks (`res.json`, `res.ok`, `res.okMeta`) so schema arguments must be:
- bridge-mode schema-name `String`, or
- typed `Schema<_>` descriptor.

Other non-schema typed placeholders are rejected.

Violations emit `E4004` with `security` + `schema` tags.

## Why it exists

Existing strict mode enforced argument presence/shape but still allowed unrelated typed values as schema descriptors. This narrowing closes that permissive gap while preserving bridge-mode string schemas.

## How it works internally

In `enforce_json_response_schema_requirements`:
1. preserve strict arity/status checks,
2. preserve invalid primitive/secret/untrusted schema rejection,
3. add descriptor narrowing to `String` or `Schema<_>`,
4. keep typed value/schema compatibility checks when schema is `Schema<T>`.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_json_response_non_schema_descriptor_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of non-schema descriptor placeholders for JSON response sinks,
  - deterministic schema-tagged diagnostics.
- Constraint:
  - bridge-mode string schema descriptors remain supported until schema values are fully first-class.

## Failure modes and diagnostics

Example:
- `res.json(pathSafeValue, payload)` ->
  - `E4004`: json response schema argument must be `String` or `Schema<_>`.

## Example usage

```ut
fn ok(schema: Schema<Int>) effects { net } -> Int {
  res.json(schema, 1);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: previously accepted non-schema typed placeholders now fail semantic checks.
- Next:
  - gradually retire bridge-mode string schemas once typed schema values are fully available in the language surface.
