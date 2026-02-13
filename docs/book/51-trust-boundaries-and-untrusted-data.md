# 51 Trust Boundaries and Untrusted Data

This chapter tracks M3 trust-boundary status and the implementation target.

## What it is

A compiler-enforced model where untrusted input cannot flow into sensitive sinks without explicit validation/gating.

## Why it exists

Without explicit trust boundaries, backend code relies on conventions that AI-generated code can easily violate. Untrusted<T> needs compile-time checks, not style-only guidance.

## How it works internally

Current state in this slice:
- effect infrastructure required for boundary enforcement is implemented,
- boundary-specific type and sink rules are planned but not yet enforced in code.

Planned boundary model:
- `Untrusted<T>` source types at HTTP/env/raw decode boundaries,
- schema gate (`req.json(schema)`) to produce trusted typed values,
- typed sinks (`SqlQuery`, `HtmlSafe`, `UrlSafe`, `PathSafe`) rejecting untrusted values.

## Inputs/outputs and constraints

- Input (planned): untrusted request/env/raw payload values.
- Output (planned): trusted typed values after schema/validator gates.
- Constraint: this chapter describes target behavior; implementation starts in the next M3 boundary slice.

## Failure modes and diagnostics

Planned diagnostics include:
- untrusted-to-sink flow rejection,
- missing gate rejection,
- unsafe secret/untrusted encoding/logging paths.

## Example usage

Target pattern:

```ut
schema CreateUserRequest {
  email: Email = validate.email
}

fn handler() effects { net } -> Int {
  let req = req.json(CreateUserRequest)?;
  1
}
```

## Tradeoffs and next steps

- Tradeoff: boundary checks are not yet active in compiler semantics.
- Next:
  - introduce `Untrusted<T>` and gate primitives,
  - enforce typed sink contracts,
  - add golden tests proving rejected unsafe flows.
