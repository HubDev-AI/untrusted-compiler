# 163 M7 Slice: json.encode Schema-Argument Hardening

This chapter documents a focused M7 hardening step for `json.encode` schema contracts.

## What it is

Added semantic checks for `json.encode`:
- call shape must be `json.encode(schema, value)`,
- schema argument rejects numeric/boolean/untrusted/secret payloads,
- when schema type is known (`Schema<T>`), encoded value must match `T`.

Violations emit `E4001` with `security` + `schema` tags.

## Why it exists

`json.encode` is a core serialization boundary. Accepting non-schema placeholders weakens strict encoding guarantees and allows ambiguous runtime behavior.

## How it works internally

In semantic trust-gate enforcement:
1. detect `json.encode` calls,
2. enforce exact two-argument shape,
3. validate schema argument shape,
4. if schema is typed (`Schema<T>`), validate value compatibility,
5. emit deterministic diagnostics for invalid encoding calls.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_json_encode_schema_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/ailang-core/tests/c_backend.rs`
    - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of malformed `json.encode` schema arguments,
  - schema-tagged diagnostics for CLI/editor tooling.
- Constraint:
  - this slice hardens `json.encode`; full `json.decode` schema/shape enforcement remains a separate follow-up.

## Failure modes and diagnostics

Example:
- `json.encode(1, value)` ->
  - `E4001`: json.encode schema argument is invalid.

## Example usage

```ailang
fn useJson(schema: Schema<Int>) -> Int {
  json.encode(schema, 3);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: placeholder `json.encode` schema arguments used in early bootstrap fixtures now fail semantic checks.
- Next:
  - enforce explicit `Schema<_>` descriptors for `json.encode` schema arguments (`171-m7-json-encode-schema-descriptor-hardening.md`).
  - apply equivalent strict schema-shape enforcement to `json.decode` call contracts.
