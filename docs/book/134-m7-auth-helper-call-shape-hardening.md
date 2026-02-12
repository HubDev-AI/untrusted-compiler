# 134 M7 Slice: Auth Helper Call-Shape Hardening

This chapter documents a focused M7 bridge-contract hardening step for auth helper intrinsics.

## What it is

Added semantic call-shape enforcement for auth helpers:
- `auth.require` must be called with exactly one argument of type `Ctx`.
- `auth.requireRole` must be called with exactly two arguments: `(Ctx, String)`.

Malformed calls now emit `E4001` with the `security` tag and canonical fix notes.

## Why it exists

Auth helpers are security boundaries, so malformed calls should fail deterministically at semantic-check time instead of drifting into runtime bridge behavior. Typed call shapes keep auth wiring explicit and auditable.

## How it works internally

In semantic trust/sink contract enforcement:
1. added `enforce_auth_helper_call_shapes(...)`,
2. targeted canonical `auth.require` / `auth.requireRole` intrinsic names,
3. enforced arity and argument-type checks (`Ctx`, `String`),
4. emitted deterministic `E4001` diagnostics with usage notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_auth_require_argument_type.ai`
    - `invalid_auth_require_role_missing_arg.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
  - C-lowering integration coverage updates:
    - `compiler/ailang-core/tests/c_backend.rs`
    - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of malformed auth-helper invocations,
  - tagged diagnostics consumable by future editor quick-fix flows.
- Constraint:
  - this slice hardens semantic contracts only; runtime ABI symbols were unchanged.

## Failure modes and diagnostics

Examples:
- `auth.require(1)` ->
  - `E4001`: auth.require argument must be `Ctx`.
- `auth.requireRole(ctx)` ->
  - `E4001`: auth.requireRole expects `(ctx, role)` arguments.

## Example usage

```ailang
fn ensureAdmin(ctx: Ctx) -> Int {
  auth.require(ctx);
  auth.requireRole(ctx, "admin");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: previously permissive placeholder-style auth helper calls now fail semantic checks.
- Next:
  - apply the same strict call-shape hardening to remaining security-sensitive helpers where argument contracts are still loose.
