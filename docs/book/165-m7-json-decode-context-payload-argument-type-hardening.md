# 165 M7 Slice: json.decode Context/Payload Argument-Type Hardening

This chapter documents the follow-up M7 hardening step for `json.decode` argument typing.

## What it is

Extended `json.decode` semantic checks so:
- argument 1 must be `Ctx`,
- argument 3 must be `Untrusted<Bytes>`.

Violations emit `E4001` with `security` + `schema` tags.

## Why it exists

`json.decode` is a trust-boundary helper. Requiring explicit `Ctx` and raw `Untrusted<Bytes>` input keeps decode flows consistent with budget-aware, boundary-safe API contracts.

## How it works internally

In `enforce_json_decode_helper_signature`:
1. detect `json.decode` call sites,
2. run existing arity/schema checks,
3. validate argument 1 (`Ctx`) and argument 3 (`Untrusted<Bytes>`),
4. emit deterministic tagged diagnostics when types are invalid.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_json_decode_context_argument_type.ut`
    - `invalid_json_decode_raw_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
  - integration fixture alignment:
    - `compiler/sec4-core/tests/c_backend.rs`
    - `compiler/sec4-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of invalid `json.decode` context/raw argument types,
  - tagged diagnostics suitable for CLI/editor security surfacing.
- Constraint:
  - this slice validates helper argument types; deeper decode-result typing flow checks remain separate.

## Failure modes and diagnostics

Examples:
- `json.decode(1, schema, raw)` ->
  - `E4001`: json.decode first argument must be `Ctx`.
- `json.decode(ctx, schema, 3)` ->
  - `E4001`: json.decode raw argument must be `Untrusted<Bytes>`.

## Example usage

```ut
fn useJson(ctx: Ctx, schema: Schema<Int>, raw: Untrusted<Bytes>) -> Int {
  json.decode(ctx, schema, raw);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: bridge fixtures that used placeholder numeric context/payload arguments required updates.
- Next:
  - tighten decode-side type propagation checks where decoded values flow into typed sinks.
