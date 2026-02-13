# 171 M7 Slice: json.encode Schema-Descriptor Hardening

This chapter documents a focused M7 hardening step for `json.encode` schema descriptor typing.

## What it is

Extended `json.encode` checks so:
- call shape remains `json.encode(schema, value)`,
- schema argument must be an explicit `Schema<_>` descriptor.

Violations emit `E4001` with `security` + `schema` tags.

## Why it exists

Rejecting only numeric/secret placeholders was still too permissive. Requiring `Schema<_>` prevents ambiguous non-schema values (for example `String`) from being treated as valid encode schemas.

## How it works internally

In `enforce_json_encode_helper_signature`:
1. preserve existing shape and unsafe-schema checks,
2. require `schema_value_type(schema)` to resolve (that is, schema type `Schema<T>`),
3. emit a dedicated diagnostic if schema descriptor typing is missing,
4. keep value-vs-schema compatibility checks when `Schema<T>` is present.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_json_encode_non_schema_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of non-`Schema<_>` encode schema arguments,
  - stable schema-tagged diagnostics for tooling.
- Constraint:
  - this slice hardens `json.encode`; corresponding strict schema-descriptor checks for `json.decode` remain separate.

## Failure modes and diagnostics

Example:
- `json.encode("schema", value)` ->
  - `E4001`: json.encode schema argument must be `Schema<_>`.

## Example usage

```ailang
fn encodeUser(schema: Schema<Int>) -> Json {
  json.encode(schema, 1)
}
```

## Tradeoffs and next steps

- Tradeoff: bridge-stage code that passed string schema names to `json.encode` now fails semantic checks.
- Next:
  - apply the same explicit `Schema<_>` descriptor requirement to `json.decode`.
