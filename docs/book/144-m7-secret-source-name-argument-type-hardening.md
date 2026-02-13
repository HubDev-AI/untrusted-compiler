# 144 M7 Slice: Secret Source Name-Argument Type Hardening

This chapter documents a focused M7 hardening step that enforces secret-name typing for `secrets.get`.

## What it is

Added semantic type enforcement for the `secrets.get` name argument:
- name input must be `String` in both compact and context-first forms.

Non-string name values now emit `E4001` with `security` + `secret` tags.

## Why it exists

`secrets.get` is a secret boundary entrypoint. Without strict name typing, placeholder/non-string values can pass semantic checks and weaken API clarity at trust boundaries.

## How it works internally

In `enforce_secret_source_call_shapes(...)`:
1. kept existing arity and context checks,
2. computed name index (`1` for compact form, `2` for context-first form),
3. validated `arg_types[name_index]` is `String`,
4. emitted deterministic `E4001` diagnostics with found-type and canonical usage note.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_secret_get_name_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of non-string `secrets.get` names,
  - secret-tagged diagnostics aligned with policy-first tooling.
- Constraint:
  - this slice tightens semantic checks only; runtime ABI and C lowering remain unchanged.

## Failure modes and diagnostics

Example:
- `secrets.get(secretsCap, 1)` ->
  - `E4001`: secret source name argument must be `String`.

## Example usage

```ailang
fn readApiToken(sec: SecretsCap) effects { secrets.read } -> Int {
  secrets.get(sec, "API_TOKEN");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: placeholder/non-string secret-name inputs now fail semantic analysis.
- Next:
  - continue applying explicit payload typing to remaining security helper arguments where type assumptions are still implicit.
