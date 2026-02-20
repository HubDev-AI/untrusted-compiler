# 1072 M39 Slice: LASM Postgres Prepared QueryOne via Row-to-JSON

This slice upgrades LASM Postgres `db.queryOne` execution from SQL literal-substitution to prepared parameter execution.

## What changed

1. Removed SQL-literal rendering path for `db.queryOne` execution.
2. Added prepared `query_opt` execution for `db.queryOne` using typed `ToSql` parameter refs.
3. Added deterministic `queryOne` row materialization wrapper:
   - runtime executes `SELECT row_to_json(_sec4_row)::text ... LIMIT 1`
   - first-row payload is parsed back into JSON for deterministic response materialization.
4. Preserved deterministic placeholder-arity validation (`$N` vs provided params) and deterministic single-statement guidance on prepared-execution multi-statement misuse.

## Why

Prepared execution is the correct DB-client behavior and removes SQL literal substitution from read paths.

The row-to-json wrapper keeps `db.queryOne` output deterministic while avoiding ad-hoc per-column type coercion logic.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
2. `cargo test -p sec4 --test commands run_command_rejects_postgres_adapter_without_dsn`
