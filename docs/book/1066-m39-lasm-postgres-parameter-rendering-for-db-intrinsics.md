# 1066 M39 Slice: LASM Postgres Parameter Rendering for DB Intrinsics

This slice adds real parameterized query handling for LASM Postgres DB intrinsic execution.

## What changed

1. Added deterministic Postgres params parsing helper for `sql.q(..., params)` values:
   - supports JSON arrays (`[42,true,3.25]`) as positional `$1..$N` params
   - supports JSON scalar payloads as a single param
   - preserves backward compatibility (`"0"` / empty => no params)
2. Added deterministic SQL literal rendering for params:
   - `null` -> `NULL`
   - booleans -> `TRUE` / `FALSE`
   - numeric strings -> unquoted numeric literals
   - other strings -> single-quoted with `'` escaping
3. Wired rendered queries into LASM Postgres intrinsic execution paths:
   - `db.exec`
   - `db.execTx`
   - `db.queryOne`
4. Updated Postgres commands integration route fixture to use `$1/$2/$3` placeholders with JSON-array params and assert typed row results.

## Why

Before this slice, Postgres execution ignored `params` in LASM runtime paths.

Parameter rendering closes that gap so `sql.q(template, params)` behaves as a real executable DB query surface for LASM alpha routes.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
2. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
