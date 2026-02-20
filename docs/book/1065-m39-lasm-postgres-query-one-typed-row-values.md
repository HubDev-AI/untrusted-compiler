# 1065 M39 Slice: LASM Postgres queryOne Typed Row Values

This slice improves LASM Postgres `db.queryOne` response fidelity by materializing row column values as typed JSON values instead of string-only cells.

## What changed

1. Added typed value materialization for Postgres `SimpleQueryRow` cells in LASM runtime:
   - booleans (`t`/`f`, `true`/`false`) -> JSON booleans
   - integers/floats -> JSON numbers
   - JSON object/array text payloads -> parsed JSON values
   - null -> JSON null (existing row-null path)
   - everything else -> JSON string fallback
2. Kept `db.queryOne` response envelope shape stable (`row` remains serialized JSON payload string), but row contents now preserve primitive types.
3. Extended Postgres LASM command integration coverage to assert typed row values (`value`, `enabled`, `ratio`) in returned row payload.

## Why

String-only row cells lose type information and make downstream consumers re-parse values manually.

Typed row materialization improves real DB client behavior for LASM alpha without breaking response envelope contracts.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
2. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
