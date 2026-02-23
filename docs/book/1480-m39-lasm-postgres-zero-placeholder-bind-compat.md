# M39: LASM Postgres zero-placeholder bind compatibility

## What changed

Postgres runtime bind behavior is now explicitly placeholder-aware:

- `db.exec` and `db.execTx` only use prepared+bind path when SQL has placeholders,
- `db.queryOne` binds an empty parameter slice when placeholder count is zero.

## Why

After arity hardening, zero-placeholder SQL with legacy params payloads needed runtime-path alignment with the documented compatibility contract (extra params ignored when no placeholders exist). This patch removes accidental bind mismatches on Postgres runtime paths.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --bin sec4 postgres_parameter_arity_allows_extra_params_when_sql_has_no_placeholders`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
