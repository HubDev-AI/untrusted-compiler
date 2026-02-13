# 141 M7 Slice: Secret Source Context-Argument Type Hardening

This chapter documents a focused M7 follow-up that tightens context-first typing for `secrets.get`.

## What it is

Added semantic type enforcement for context-first secret source calls:
- `secrets.get(ctx, secretsCap, name)` requires argument 1 to be `Ctx`.

Violations now emit `E4001` with `security` + `secret` tags.

## Why it exists

Secret source call-shape checks already constrained arity, but context-first form still accepted non-context placeholders in slot 1. This update keeps secret boundary entrypoints consistent with typed context semantics.

## How it works internally

In `enforce_secret_source_call_shapes(...)`:
1. kept existing arity checks,
2. added context-first branch for the 3-argument form,
3. validated `arg_types[0]` is `Ctx`,
4. emitted deterministic `E4001` diagnostics with found-type and canonical usage notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_secret_get_context_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of invalid context-first `secrets.get` calls,
  - secret-tagged diagnostics aligned with security-first tooling surfaces.
- Constraint:
  - this slice updates semantic validation only; runtime ABI and C lowering are unchanged.

## Failure modes and diagnostics

Example:
- `secrets.get(1, secretsCap, "API_TOKEN")` ->
  - `E4001`: secret source context argument must be `Ctx`.

## Example usage

```ut
fn readSecret(ctx: Ctx, sec: SecretsCap) effects { secrets.read } -> Int {
  secrets.get(ctx, sec, "API_TOKEN");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: context-first `secrets.get` calls with non-`Ctx` slot-1 values now fail semantic analysis.
- Next:
  - apply equivalent context-first type enforcement to `secrets.reveal(ctx, secretsCap, secret)` for full secret-helper parity.
