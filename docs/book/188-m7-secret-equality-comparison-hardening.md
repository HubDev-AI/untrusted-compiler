# 188 M7 Slice: Secret Equality Comparison Hardening

This chapter documents a focused M7 hardening step for secret comparisons.

## What it is

Direct equality/inequality comparisons (`==`, `!=`) on `Secret<_>` operands are now rejected at compile time.

Violations emit `E1006` diagnostics tagged with `security` + `secret`.

## Why it exists

The security baseline requires constant-time comparison for secrets to avoid timing side-channel leaks. This slice makes accidental direct secret comparison a compile-time error.

## How it works internally

In binary comparison analysis (`ExprKind::Binary`):
1. when operator is `==` or `!=`,
2. and either operand type contains `Secret<_>`,
3. emit `E1006` with operand-type notes and constant-time compare guidance.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture: `invalid_secret_equality_comparison.ut`
  - tag test coverage in `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection for direct secret equality checks.
- Constraint:
  - this slice introduces the guard and guidance only; constant-time helper intrinsics are not introduced yet.

## Failure modes and diagnostics

Example:
- `a == b` where both are `Secret<String>` -> `E1006`: secret equality comparison is forbidden.

## Example usage

```ut
fn bad(a: Secret<String>, b: Secret<String>) -> Bool {
  a == b
}
```

## Tradeoffs and next steps

- Tradeoff: secret comparisons now require explicit helper usage once constant-time compare APIs are available.
- Next:
  - `crypto.ctEq` helper bridge and typed semantic contracts are documented in `189-m7-crypto-cteq-intrinsic-bridge.md`.
