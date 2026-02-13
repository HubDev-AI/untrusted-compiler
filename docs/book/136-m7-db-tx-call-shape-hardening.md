# 136 M7 Slice: db.tx Call-Shape Hardening

This chapter documents a focused M7 bridge-contract hardening step for `db.tx`.

## What it is

Added semantic argument-shape enforcement for `db.tx`:
- accepts only `(dbCap)` or `(ctx, dbCap)`.

Malformed calls now emit `E4001` with `security` + `capability` tags.

## Why it exists

`db.tx` is a capability-sensitive database boundary. Enforcing explicit compact/context-first call shapes keeps transaction entrypoints consistent with the rest of the capability contract surface.

## How it works internally

In semantic trust/sink contract enforcement:
1. added `enforce_db_tx_call_shape(...)`,
2. targeted canonical `db_tx` / `db.tx` names,
3. enforced allowed arities (`1` or `2`),
4. emitted deterministic `E4001` diagnostics with canonical usage notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_db_tx_argument_shape.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of malformed `db.tx` argument shapes,
  - capability-tagged diagnostics suitable for tooling surfaces.
- Constraint:
  - this slice hardens call shape only; runtime ABI behavior is unchanged.

## Failure modes and diagnostics

Example:
- `db.tx(1, db, 2)` ->
  - `E4001`: db.tx call has invalid argument shape.

## Example usage

```ut
fn startTx(ctx: Ctx, db: DbCap) effects { db.tx } -> Int {
  db.tx(ctx, db);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: permissive multi-argument placeholder patterns now fail at semantic stage.
- Next:
  - continue tightening capability-helper argument typing for compact vs context-first forms where only arity is currently enforced.
