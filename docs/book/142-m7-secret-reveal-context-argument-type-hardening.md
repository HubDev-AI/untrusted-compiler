# 142 M7 Slice: Secret Reveal Context-Argument Type Hardening

This chapter documents a focused M7 follow-up that tightens context-first typing for `secrets.reveal`.

## What it is

Added semantic type enforcement for context-first secret reveal calls:
- `secrets.reveal(ctx, secretsCap, secret)` requires argument 1 to be `Ctx`.

Violations now emit `E4001` with `security` + `secret` tags.

## Why it exists

Secret reveal call-shape checks already constrained arity, but context-first form still accepted non-context placeholders in slot 1. This update closes that gap so secret-unsealing call contracts remain explicit and auditable.

## How it works internally

In `enforce_secret_reveal_call_shapes(...)`:
1. kept existing arity checks,
2. added a context-first branch for 3-argument reveal calls,
3. validated `arg_types[0]` is `Ctx`,
4. emitted deterministic `E4001` diagnostics with found-type and canonical usage notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_secret_reveal_context_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of invalid context-first `secrets.reveal` calls,
  - secret-tagged diagnostics aligned with security-first tooling.
- Constraint:
  - policy-forbidden `secrets.reveal` diagnostics (`E2002`) still appear by default unless allowlisted; this slice adds an additional shape/type diagnostic, not a policy override.

## Failure modes and diagnostics

Example:
- `secrets.reveal(1, secretsCap, token)` ->
  - `E4001`: secret reveal context argument must be `Ctx`.

## Example usage

```ut
fn revealOne(ctx: Ctx, sec: SecretsCap, token: Secret<String>)
  effects { secrets.reveal }
  -> Int
{
  secrets.reveal(ctx, sec, token);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: context-first reveal calls with non-`Ctx` slot-1 values now fail semantic checks in addition to policy checks.
- Next:
  - continue extending context-first typed slot checks across remaining capability-backed helper families to complete M7 contract parity.
