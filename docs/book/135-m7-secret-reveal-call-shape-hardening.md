# 135 M7 Slice: Secret Reveal Call-Shape Hardening

This chapter documents a focused M7 bridge-contract hardening step for `secrets.reveal`.

## What it is

Added semantic argument-shape enforcement for `secrets.reveal`:
- accepts only `(secretsCap, secret)` or `(ctx, secretsCap, secret)`.

Malformed calls now emit `E4001` with `security` + `secret` tags.

## Why it exists

`secrets.reveal` is the highest-risk secret API in the v0 surface. Tightening its call shape reduces ambiguous helper usage and keeps policy/audit signals aligned with explicit capability-bearing reveal paths.

## How it works internally

In semantic trust/sink contract enforcement:
1. added `enforce_secret_reveal_call_shapes(...)`,
2. targeted canonical `secret_reveal` / `secrets.reveal` names,
3. enforced allowed arities (`2` or `3`),
4. emitted deterministic `E4001` diagnostics with canonical usage notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_secret_reveal_missing_secret.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
  - security metadata/audit fixtures:
    - `compiler/sec4-core/tests/security_map.rs`
    - `compiler/sec4-core/tests/sec_audit.rs`
- Outputs:
  - compile-time rejection of malformed reveal helper calls,
  - preserved deterministic `security_map`/`sec4 audit` coverage using canonical reveal shape.
- Constraint:
  - this slice changes semantic contracts only; runtime ABI symbols remain unchanged.

## Failure modes and diagnostics

Example:
- `secrets.reveal(sec)` ->
  - `E4001`: secret reveal call has invalid argument shape.

## Example usage

```ut
fn unsafeUnseal(ctx: Ctx, sec: SecretsCap, token: Secret<String>)
  effects { secrets.reveal }
  -> Int
{
  secrets.reveal(ctx, sec, token);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: single-argument reveal placeholders now fail earlier at semantic-check time.
- Next:
  - continue tightening remaining security-sensitive helper signatures where capability and payload roles are still inferred implicitly.
