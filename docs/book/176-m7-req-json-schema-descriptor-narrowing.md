# 176 M7 Slice: req.json Schema-Descriptor Narrowing

This chapter documents a focused M7 hardening step for `req.json` schema descriptor acceptance.

## What it is

Extended `req.json` schema-gate checks so schema arguments must be:
- bridge-mode schema-name `String`, or
- typed `Schema<_>` descriptor.

Other non-primitive placeholders (for example `PathSafe`) are now rejected.

Violations emit `E4001` with `security` + `schema` tags.

## Why it exists

Previous checks rejected only numeric/boolean/secret/untrusted values. That still allowed unrelated typed values to be used as schema descriptors, which weakens the request decode contract.

## How it works internally

In `enforce_trust_gate_requirements` for `req.json`:
1. preserve existing arity and invalid-shape checks,
2. keep bridge compatibility (`String` descriptors),
3. allow typed schema descriptors via `Schema<T>`,
4. reject all other descriptor types with dedicated diagnostics.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_req_json_non_schema_descriptor_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of non-schema descriptor placeholders for `req.json`,
  - deterministic schema-tagged diagnostics for CLI/editor tooling.
- Constraint:
  - bridge-mode string schema names remain supported until schema value declarations are fully modeled.

## Failure modes and diagnostics

Example:
- `req.json(pathSafeValue)` ->
  - `E4001`: req.json schema argument must be `String` or `Schema<_>`.

## Example usage

```ut
fn createUser() effects { net } -> Int {
  req.json("CreateUserRequest");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: previously accepted non-schema typed placeholders now fail semantic checks.
- Next:
  - eventually retire string descriptors and move to typed schema values only once schema declarations are first-class value-level symbols.
