# 172 M7 Slice: json.decode Schema-Descriptor Hardening

This chapter documents a focused M7 hardening step for `json.decode` schema descriptor typing.

## What it is

Extended `json.decode` checks so:
- call shape remains `json.decode(ctx, schema, raw)`,
- schema argument must be an explicit `Schema<_>` descriptor.

Violations emit `E4001` with `security` + `schema` tags.

## Why it exists

Rejecting only numeric/secret placeholders was still permissive for decode schema arguments. Requiring `Schema<_>` prevents ambiguous non-schema values (for example `String`) from being accepted as decode gates.

## How it works internally

In `enforce_json_decode_helper_signature`:
1. preserve existing shape and unsafe-schema checks,
2. require `schema_value_type(schema)` to resolve (schema type `Schema<T>`),
3. emit a dedicated schema-descriptor diagnostic when missing,
4. keep existing context/raw argument checks unchanged.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_json_decode_non_schema_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of non-`Schema<_>` decode schema arguments,
  - stable schema-tagged diagnostics for tooling.
- Constraint:
  - this slice hardens decode schema descriptors only; req.json schema descriptor hardening remains separate.

## Failure modes and diagnostics

Example:
- `json.decode(ctx, "schema", raw)` ->
  - `E4001`: json.decode schema argument must be `Schema<_>`.

## Example usage

```ailang
fn decodeUser(ctx: Ctx, schema: Schema<Int>, raw: Untrusted<Bytes>) -> Int {
  json.decode(ctx, schema, raw);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: bridge-stage decode samples that used string schema names now fail semantic checks.
- Next:
  - apply explicit `Schema<_>` descriptor rules to `req.json` schema-gate arguments.
