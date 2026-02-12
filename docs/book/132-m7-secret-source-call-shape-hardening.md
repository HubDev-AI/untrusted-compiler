# 132 M7 Slice: Secret Source Call-Shape Hardening

This chapter documents a focused M7 bridge-contract hardening step for secret source calls.

## What it is

Added semantic argument-shape enforcement for `secrets.get`:
- allowed forms: `(secretsCap, name)` or `(ctx, secretsCap, name)`.

Malformed calls now emit `E4001` with `security` + `secret` tags.

## Why it exists

`secrets.get` is the typed secret boundary. Permitting calls without an explicit secret name weakens boundary clarity and makes policy/audit behavior less deterministic.

This slice makes source intent explicit while keeping runtime behavior unchanged.

## How it works internally

In semantic trust/sink contract enforcement:
1. added `enforce_secret_source_call_shapes(...)`,
2. targeted canonical `secret_read`/`secrets.get` calls,
3. validated compact and context-first arities,
4. emitted deterministic `E4001` diagnostics with canonical fix notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_secret_get_missing_name.ai`
    - updated `valid_capabilities_effects.ai`
  - diagnostic-tag coverage in `compiler/ailang-core/tests/diagnostic_tags.rs`
  - secret-read CLI integration test (`compiler/ailang-cli/tests/json_output.rs`)
- Outputs:
  - compile-time rejection of malformed `secrets.get` argument shapes,
  - updated fixtures/integration to explicit secret-name usage (`"TOKEN"`).
- Constraint:
  - this slice is call-shape hardening only; stricter secret-name typing/policy semantics can be layered later.

## Failure modes and diagnostics

Examples:
- `secrets.get(secretsCap)` ->
  - `E4001`: secret source call has invalid argument shape.

## Example usage

```ailang
fn readSecret(sec: SecretsCap) effects { secrets.read } -> Int {
  secrets.get(sec, "TOKEN");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: older placeholder calls without explicit names now fail fast.
- Next:
  - tighten `secrets.reveal` call-shape similarly,
  - keep diagnostics aligned with policy-driven reveal restrictions.
