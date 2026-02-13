# 120 M7 Slice: `req.json` Schema-Argument Contract Hardening

This chapter documents a focused M7 schema-gate hardening step for request decoding.

## What it is

Extended `req.json(...)` semantic validation:
- still rejects missing schema argument,
- now also rejects invalid schema argument types:
  - numeric,
  - boolean,
  - `Untrusted<_>`,
  - `Secret<_>`.

## Why it exists

Previously `req.json` checked only argument presence. That allowed obviously invalid schema placeholders (`req.json(123)`) to pass semantic checks. This slice makes schema-gated decode intent explicit and prevents weak call patterns early.

## How it works internally

In semantic intrinsic gate checks:
1. detect `req.json` calls,
2. if no args -> existing missing-schema diagnostic (`E4001`),
3. if arg exists -> validate schema-argument type shape and emit `E4001` on invalid forms.

Diagnostics for this rule are tagged with `security` and `schema` for tooling/LSP consumers.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture `invalid_req_json_schema_arg_type.ai`
  - diagnostic tag tests
- Outputs:
  - compile-time rejection of invalid schema gate arguments
  - tagged diagnostics for editor integration
- Constraint:
  - this is shape-level validation; full schema symbol/type binding rules can be tightened further in later slices.

## Failure modes and diagnostics

Example:
- `req.json(123)` -> `E4001` (`schema gate argument is invalid`), with notes explaining expected schema descriptor usage.

## Example usage

```ailang
fn createUser() effects { net } -> Int {
  req.json("CreateUserRequest");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: accepted schema descriptors are still permissive (`Unknown`/string-style descriptors remain allowed for bridge stage).
- Next:
  - narrow accepted descriptors to bridge-name `String` or typed `Schema<_>` only (`176-m7-req-json-schema-descriptor-narrowing.md`),
  - continue tightening req/res call contracts,
  - then align runtime decode behavior and budgets with these semantic guarantees.
