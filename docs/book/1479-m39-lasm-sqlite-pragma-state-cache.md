# M39: LASM sqlite pragma state cache

## What changed

LASM dynamic runtime now caches effective sqlite pragma settings in state at bootstrap:

- `db_sqlite_journal_mode`
- `db_sqlite_synchronous`

Usage updates:

- sqlite connect/reconnect paths now consume cached pragma values,
- `/db/records` telemetry uses cached values instead of re-resolving env per request.

## Why

After adding pragma telemetry, runtime code could repeatedly resolve env values and re-emit warnings on invalid env each time telemetry was materialized. Caching settings once keeps runtime behavior deterministic while avoiding repeated env parsing/warning noise.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --bin sec4 positional_arity_validation_allows_extra_params_when_sql_has_no_placeholders`
- `cargo test -p sec4 --bin sec4 postgres_parameter_arity_allows_extra_params_when_sql_has_no_placeholders`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
