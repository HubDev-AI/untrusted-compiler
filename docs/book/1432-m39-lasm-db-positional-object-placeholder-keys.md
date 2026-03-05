# M39: LASM DB Positional Object Placeholder Keys

Date: 2026-02-22  
Milestone: M39 (DB runtime usability hardening)

## What Changed

- Extended positional-object SQL param parsing for both Postgres and SQLite adapters.
- Positional keys now accept placeholder-style prefixes:
  - `$1`, `$2`, ...
  - `?1`, `?2`, ...
- Plain numeric keys (`\"1\"`, `\"2\"`) remain supported.

## Why

Many generated SQL payload transformers emit placeholder-shaped keys instead of bare numeric keys. Supporting these forms avoids unnecessary fallback-to-text behavior and keeps positional binding deterministic.

## Result

- `{\"$2\": \"alice\"}` resolves to `[null, \"alice\"]`.
- `{\"?3\": 7}` resolves to `[null, null, 7]`.
- Non-positional keys still follow fallback behavior and do not break compatibility.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 positional_object_params_accept_placeholder_prefixed_keys`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
