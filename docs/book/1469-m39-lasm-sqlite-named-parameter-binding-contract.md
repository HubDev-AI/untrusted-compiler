# M39: LASM sqlite named-parameter binding contract hardening

## What changed

LASM sqlite runtime named-parameter execution now resolves bindings against prepared statement placeholders before query execution.

Runtime behavior:

- supports named placeholder prefix variants from SQL template (`:name`, `@name`, `$name`),
- validates each statement placeholder has a matching param entry,
- rejects extra named params not present in SQL template,
- rejects named-param payloads when SQL uses unnamed placeholders (`?` style).

## Why

Previous named-param execution forwarded canonicalized `:name` bindings directly, which could drift from actual SQL placeholder names and defer contract errors to sqlite internals. This change makes parameter-contract validation explicit and deterministic in LASM runtime.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --bin sec4 named_bindings_match_statement_prefix_variants`
- `cargo test -p sec4 --bin sec4 classify_db_runtime_named_param_errors_as_validation`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_sqlite_records_and_query_one_when_db_intrinsics_are_used`
