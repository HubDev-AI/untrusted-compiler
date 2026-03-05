# M39: LASM SQLite Named Object Params

Date: 2026-02-22  
Milestone: M39 (DB runtime client behavior hardening)

## What Changed

- Added named-object SQL parameter parsing for LASM SQLite runtime.
- SQLite parser now supports named keys in object params:
  - plain (`name`)
  - prefixed (`:name`, `@name`, `$name`)
- Named bindings are canonicalized and passed into sqlite statement execution/query paths as named parameters.
- Existing positional parsing and positional-object expansion behavior remains in place.

## Why

SQLite supports named SQL parameters, but runtime param parsing previously forced object params into positional/fallback-only behavior. Supporting named-object bindings makes sqlite adapter behavior closer to real client expectations.

## Result

- SQLite DB intrinsics can now bind named object params directly.
- Dispatch pre-parse path (outside lock) now covers both positional and named sqlite param sets.
- Invalid object keys still keep deterministic fallback behavior.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 named_object_params_accept_prefixed_and_plain_names`
- `cargo test -p sec4 positional_object_params_expand_with_null_fill`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
