# M39: LASM Postgres Named Object Params

Date: 2026-02-22  
Milestone: M39 (DB runtime client behavior hardening)

## What Changed

- Added template-aware Postgres SQL param parsing for object payloads with named keys:
  - plain (`name`)
  - prefixed (`:name`, `@name`, `$name`)
- Added deterministic query-template rewrite for Postgres named placeholders:
  - supports `:name`, `@name`, `$name` in SQL templates,
  - rewrites to positional `$1..$N` in first-appearance order,
  - reuses the same positional index for repeated named placeholders.
- Rewrite scanner is SQL-shape-aware:
  - skips single-quoted literals,
  - skips line/block comments,
  - skips dollar-quoted SQL blocks,
  - preserves Postgres cast syntax (`::type`) without false placeholder rewrites.
- Wired dispatch/runtime integration for `db.exec`, `db.execTx`, and `db.queryOne`:
  - Postgres paths now consume rewritten template + parsed params,
  - missing named parameters return deterministic validation envelopes (`DB.EXEC_INVALID`, `DB.EXEC_TX_INVALID`, `DB.QUERY_ONE_INVALID`).

## Why

SQLite already supported named object params while Postgres accepted only positional object forms. That mismatch forced AI-generated/object-shaped payloads into fallback behavior for Postgres paths and limited real client ergonomics.

## Result

- Postgres runtime now supports named-object params with predictable binding semantics.
- Existing positional params (`1`, `$1`, `?1`, arrays) continue to work unchanged.
- Runtime behavior is closer to practical DB-client expectations across sqlite/postgres adapters.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 named_object_params_rewrite_colon_placeholders`
- `cargo test -p sec4 named_object_params_skip_cast_literals_comments_and_reuse_indices`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
