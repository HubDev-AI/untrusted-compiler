# 189 M7 Slice: crypto.ctEq Intrinsic Bridge

This chapter documents the M7 bridge slice for constant-time secret comparison.

## What it is

Added `crypto.ctEq` (and underscore alias `crypto_ct_eq`) as a typed intrinsic:
- signature: `crypto.ctEq(secretA, secretB) -> Bool`
- semantic contract: both arguments must be `Secret<_>` and type-compatible
- C lowering target: `sec4_rt_crypto_ct_eq`

## Why it exists

After forbidding direct secret `==`/`!=` comparisons, the language needs a canonical constant-time comparison path. This slice provides that bridge surface and runtime symbol wiring.

## How it works internally

1. Semantic intrinsics catalog now includes `crypto.ctEq` with `Bool` return type.
2. Semantic signature checks enforce two-arg shape and `Secret<_>` argument contracts.
3. C backend intrinsic rewriting maps dotted/underscore call forms to runtime symbol `sec4_rt_crypto_ct_eq`.
4. Runtime ABI includes `bool sec4_rt_crypto_ct_eq();` stub implementation.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - `compiler/sec4-core/src/c_backend.rs`
  - runtime ABI files:
    - `runtime/c/sec4_runtime.h`
    - `runtime/c/sec4_runtime.c`
  - tests:
    - semantic fixtures/tags
    - c backend rewrite tests
    - CLI `c-bin` integration test
- Outputs:
  - typed `crypto.ctEq` compile-time contract and runtime-lowered symbol path.
- Constraint:
  - runtime implementation is currently a stub and does not yet implement real constant-time comparison semantics.

## Failure modes and diagnostics

Examples:
- `crypto.ctEq(a, b)` with non-secret args -> `E4001` argument-type diagnostics.
- incompatible secret argument types -> `E4001` compatibility diagnostic.

## Example usage

```ut
fn compare(a: Secret<String>, b: Secret<String>) -> Bool {
  crypto.ctEq(a, b)
}
```

## Tradeoffs and next steps

- Tradeoff: the API and type checks are in place before final runtime crypto semantics.
- Next:
  - replace runtime stub with verified constant-time implementation and add runtime conformance tests.
