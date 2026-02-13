# 137 M7 Slice: db.tx Context-Argument Type Hardening

This chapter documents a focused M7 follow-up that tightens context-first typing for `db.tx`.

## What it is

Added semantic type enforcement for context-first `db.tx` calls:
- when using two-argument form, argument 1 must be `Ctx`.

Violations now emit `E4001` with `security` + `capability` tags.

## Why it exists

The previous slice hardened `db.tx` arity, but `(ctx, dbCap)` still accepted non-context values in slot 1. This update keeps compact/context-first contracts explicit and prevents placeholder values from bypassing context semantics.

## How it works internally

In `enforce_db_tx_call_shape(...)`:
1. kept arity gate (`1` or `2`),
2. added a two-argument typed branch,
3. validated `arg_types[0]` is `Ctx`,
4. emitted deterministic `E4001` diagnostic with found-type note and canonical fix note.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_db_tx_context_argument_type.ut`
  - diagnostic-tag coverage:
    - `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of invalid context-first `db.tx` calls,
  - capability-tagged diagnostics aligned with existing capability enforcement flows.
- Constraint:
  - this slice does not alter runtime ABI or C lowering.

## Failure modes and diagnostics

Example:
- `db.tx(1, db)` ->
  - `E4001`: db.tx context argument must be `Ctx`
  - note: found `Int`

## Example usage

```ut
fn begin(ctx: Ctx, db: DbCap) effects { db.tx } -> Int {
  db.tx(ctx, db);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: non-`Ctx` placeholders in context-first position now fail semantic checks.
- Next:
  - extend equivalent context-first type checks across other compact/context-first capability intrinsics where slot-0 context typing is still implicit.
