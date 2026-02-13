# 164 M7 Slice: json.decode Schema-Argument Hardening

This chapter documents a focused M7 hardening step for `json.decode` schema contracts.

## What it is

Added semantic checks for `json.decode`:
- call shape must be `json.decode(ctx, schema, raw)`,
- schema argument rejects numeric/boolean/untrusted/secret payloads.

Violations emit `E4001` with `security` + `schema` tags.

## Why it exists

`json.decode` is a trust-boundary gate. Accepting placeholder or unsafe schema arguments weakens explicit schema-driven decoding guarantees.

## How it works internally

In semantic trust-gate enforcement:
1. detect `json.decode` calls,
2. enforce exact three-argument shape `(ctx, schema, raw)`,
3. validate schema argument shape,
4. emit deterministic diagnostics for malformed calls.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_json_decode_schema_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/ailang-core/tests/c_backend.rs`
    - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of malformed `json.decode` schema arguments,
  - schema-tagged diagnostics for CLI/editor tooling.
- Constraint:
  - this slice hardens schema-argument semantics; deeper `ctx/raw` typing hardening remains a follow-up.

## Failure modes and diagnostics

Examples:
- `json.decode(1, 2, 3)` ->
  - `E4001`: json.decode schema argument is invalid.
- `json.decode(schema, raw)` ->
  - `E4001`: json.decode expects `(ctx, schema, raw)` arguments.

## Example usage

```ailang
fn useJson(ctx: Ctx, schema: Schema<Int>, raw: Untrusted<Bytes>) -> Int {
  json.decode(ctx, schema, raw);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: early bootstrap fixtures using placeholder schema values now fail semantic checks.
- Next:
  - enforce explicit `Schema<_>` descriptors for `json.decode` schema arguments (`172-m7-json-decode-schema-descriptor-hardening.md`).
  - harden `json.decode` context/payload typing beyond call-shape and schema-argument validation (`165-m7-json-decode-context-payload-argument-type-hardening.md`).
