# 190 M7 Slice: security_map crypto.ctEq Tagging

This chapter documents the M7 security-map extension for `crypto.ctEq`.

## What it is

Added `security_map` coverage for `crypto.ctEq` (and underscore alias `crypto_ct_eq`):
- call tags now include `gate.crypto.ct_eq`
- call argument roles now include `left_secret` and `right_secret`
- symbol registry now includes both dotted and underscore spellings

## Why it exists

The previous slice added the intrinsic bridge and semantic checks, but audit metadata still needed a first-class marker for this secret-comparison gate. Without this tag, policy/audit tooling cannot attribute constant-time comparisons as intentional secret-safe gates.

## How it works internally

1. `call_tags_for(...)` maps `crypto.ctEq` / `crypto_ct_eq` to `gate.crypto.ct_eq`.
2. `call_arg_roles(...)` maps both arguments to secret-specific roles (`left_secret`, `right_secret`).
3. `intrinsic_symbol_registry()` publishes the same gate tag for both intrinsic name forms.
4. `security_map` tests verify:
   - tagged call emission for dotted and underscore names
   - arg-role emission at callsites
   - symbol registry coverage

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/security_map.rs`
  - `compiler/ailang-core/tests/security_map.rs`
- Output:
  - richer `security_map` metadata for constant-time secret comparisons.
- Constraint:
  - this slice is metadata-only; it does not alter semantic acceptance rules or runtime lowering.

## Failure modes and diagnostics

- If call tagging regresses, `security_map` tests fail because `gate.crypto.ct_eq` is missing from call tags.
- If role mapping regresses, tests fail because `left_secret`/`right_secret` roles are not emitted for the intrinsic call.

## Example usage

```ailang
fn same(a: Secret<String>, b: Secret<String>) -> Bool {
  crypto.ctEq(a, b)
}
```

`security_map` now records this as a gate call with secret-argument roles.

## Tradeoffs and next steps

- Tradeoff: role names are fixed string labels in metadata; they are simple and stable, but intentionally minimal.
- Next:
  - add audit rule hooks that can explicitly recognize `gate.crypto.ct_eq` as the sanctioned replacement for direct secret equality checks.
