# 145 M7 Slice: DB Sink Query-Argument Type Hardening

This chapter documents a focused M7 hardening step that enforces `SqlQuery` typing for DB sink query inputs.

## What it is

Added semantic type enforcement for DB sink query arguments:
- `db.exec` query argument must be `SqlQuery`.
- `db.execTx` query argument must be `SqlQuery`.
- `db.queryOne` query argument must be `SqlQuery`.

This applies to both compact and context-first call forms.

Violations emit `E4001` with `security` + `sink` tags.

## Why it exists

DB sink call-shape and context typing were already hardened, but query payloads could still be passed as generic trusted types. Enforcing `SqlQuery` closes that gap and matches the typed-sink baseline.

## How it works internally

In `enforce_db_query_call_shapes(...)`:
1. kept arity and context checks,
2. selected query argument index by call form,
3. delegated to `enforce_db_sink_query_type(...)`,
4. validated query argument is `SqlQuery`,
5. skipped duplicate diagnostics when query type already contains `Untrusted<_>` or `Secret<_>` (those flows are covered by sink-flow diagnostics),
6. emitted deterministic `E4001` diagnostics with found-type and canonical usage notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_db_query_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
  - semantic fixture alignment updates for capability/alias call paths so they continue to isolate capability assertions using typed `SqlQuery` values.
- Outputs:
  - compile-time rejection of non-`SqlQuery` DB sink query values,
  - sink-tagged diagnostics aligned with typed-sink policy.
- Constraint:
  - this slice affects semantic checks and fixtures only; runtime ABI and C lowering remain unchanged.

## Failure modes and diagnostics

Example:
- `db.exec(dbCap, 1)` ->
  - `E4001`: db sink query argument must be `SqlQuery`.

## Example usage

```ailang
fn writeUser(db: DbCap, query: SqlQuery) effects { db.write } -> Int {
  db.exec(db, query);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: previously accepted trusted-but-untyped DB query values now fail semantic analysis.
- Next:
  - apply equivalent strict typed payload enforcement to net (`PublicUrl`/`InternalUrl`) and filesystem (`PathSafe`) sink arguments for full sink-family parity.
