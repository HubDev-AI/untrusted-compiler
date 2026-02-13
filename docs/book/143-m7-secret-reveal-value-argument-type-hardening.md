# 143 M7 Slice: Secret Reveal Value-Argument Type Hardening

This chapter documents a focused M7 hardening step that enforces secret value typing for `secrets.reveal`.

## What it is

Added semantic type enforcement for `secrets.reveal` payload input:
- reveal value argument must be `Secret<_>`.

Non-secret payloads now emit `E4001` with `security` + `secret` tags.

## Why it exists

`secrets.reveal` is the explicit escape hatch from secret safety. Without payload typing, plain trusted values could flow through reveal paths, weakening policy clarity and audit signal quality.

## How it works internally

In `enforce_secret_reveal_call_shapes(...)`:
1. kept existing arity and context checks,
2. selected reveal payload index (`1` for compact form, `2` for context-first form),
3. validated `arg_types[value_index].contains_secret()`,
4. emitted deterministic `E4001` diagnostics with found-type and canonical usage note.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_secret_reveal_value_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of non-secret reveal payloads,
  - secret-tagged diagnostics suitable for tooling and policy review.
- Constraint:
  - policy-forbidden reveal diagnostics (`E2002`) still appear by default; this slice adds a stricter payload-type diagnostic on top.

## Failure modes and diagnostics

Example:
- `secrets.reveal(secretsCap, tokenString)` ->
  - `E4001`: secret reveal value argument must be `Secret<_>`.

## Example usage

```ut
fn revealToken(sec: SecretsCap, token: Secret<String>)
  effects { secrets.reveal }
  -> Int
{
  secrets.reveal(sec, token);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: reveal calls that passed non-secret placeholders now fail semantic checks.
- Next:
  - continue tightening remaining capability helper payload typing to match the same explicit boundary style.
