# 174 M7 Slice: db.queryOne Row-Schema Descriptor Hardening

This chapter documents a focused M7 hardening step for `db.queryOne` row-schema descriptors.

## What it is

Extended `db.queryOne` row-schema checks so:
- compact form remains `db.queryOne(capability, query, rowSchema)`,
- context-first form remains `db.queryOne(ctx, capability, query, rowSchema)`,
- row-schema argument must be an explicit `Schema<_>` descriptor.

Violations emit `E4001` with `security` + `schema` tags.

## Why it exists

Primitive placeholder rejection was not enough. Requiring typed `Schema<_>` row descriptors prevents ambiguous values (for example `String`) from being treated as valid DB row schema arguments.

## How it works internally

In `enforce_db_query_one_row_schema_type`:
1. preserve existing primitive/secret/untrusted handling,
2. require `schema_value_type(rowSchema)` to resolve (type `Schema<T>`),
3. emit dedicated non-schema diagnostics when the descriptor is missing.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture:
    - `invalid_db_query_one_non_schema_argument_type.ai`
  - fixture alignment:
    - `compiler/ailang-core/tests/fixtures/semantic/valid_capabilities_effects_ctx.ai`
    - `compiler/ailang-core/tests/fixtures/semantic/invalid_capability_mismatch.ai`
  - integration fixture alignment:
    - `compiler/ailang-cli/tests/json_output.rs`
  - diagnostic-tag coverage:
    - `compiler/ailang-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of non-`Schema<_>` row-schema arguments in `db.queryOne`,
  - deterministic schema-tagged diagnostics.
- Constraint:
  - runtime DB bridge behavior is unchanged.

## Failure modes and diagnostics

Example:
- `db.queryOne(dbCap, query, "Row")` ->
  - `E4001`: db.queryOne row schema argument must be `Schema<_>`.

## Example usage

```ailang
fn load(db: DbCap, query: SqlQuery, rowSchema: Schema<Int>) effects { db.read } -> Int {
  db.queryOne(db, query, rowSchema);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: bridge-stage code passing string row schema names now fails semantic checks.
- Next:
  - evolve row-schema typing from `Schema<_>` placeholders toward dedicated `RowSchema<T>` contracts.
