# 149 M7 Slice: db.queryOne Row-Schema Argument Hardening

This chapter documents a focused M7 hardening step for `db.queryOne(..., rowSchema)` schema-like argument validation.

## What it is

Added semantic validation for `db.queryOne` row-schema arguments:
- compact form: `db.queryOne(capability, query, rowSchema)`
- context-first form: `db.queryOne(ctx, capability, query, rowSchema)`

Numeric and boolean placeholder values are now rejected.

Violations emit `E4001` with `security` + `schema` tags.

## Why it exists

`db.queryOne` has a schema-descriptor slot for row decoding. Accepting primitive placeholder values (for example `1` or `true`) weakens the typed contract and makes invalid query wiring appear valid until later stages.

## How it works internally

In DB sink call-shape enforcement:
1. preserve existing shape/context/query checks,
2. for `db.queryOne`, resolve row-schema argument index by call form,
3. reject numeric/boolean row-schema arguments,
4. emit deterministic `E4001` diagnostics with found-type and canonical usage guidance.

To avoid duplicate diagnostics, secret/untrusted wrappers in this slot continue to rely on existing sink-flow diagnostics.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_db_query_one_row_schema_argument_type.ai`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of invalid primitive row-schema placeholders,
  - schema-tagged diagnostics for editor/CLI consumers.
- Constraint:
  - this slice enforces schema-slot shape only; no runtime DB decoding behavior changes.

## Failure modes and diagnostics

Example:
- `db.queryOne(dbCap, query, 1)` ->
  - `E4001`: db.queryOne row schema argument is invalid.

## Example usage

```ailang
fn load(db: DbCap, query: SqlQuery, rowSchema: String) effects { db.read } -> Int {
  db.queryOne(db, query, rowSchema);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: invalid primitive placeholders that previously passed semantic checks now fail early.
- Next:
  - evolve row-schema slot typing from descriptor-shape validation toward explicit `RowSchema<T>` contracts when that type surface is finalized.
