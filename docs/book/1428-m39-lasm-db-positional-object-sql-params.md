# M39: LASM DB Positional Object SQL Params

Date: 2026-02-22  
Milestone: M39 (DB runtime usability hardening)

## What Changed

- Added positional-object SQL parameter parsing for LASM Postgres runtime.
- Added positional-object SQL parameter parsing for LASM SQLite runtime.
- Object keys are interpreted as 1-based positional indices (`\"1\"`, `\"2\"`, ...), with deterministic null-fill for missing positions.
- Non-numeric object keys keep fallback behavior (single text parameter payload).

## Why

`sql.q(...)` runtime param payloads were limited to array/scalar forms. AI-generated flows and operator tooling often emit object-shaped payloads. Supporting numeric-key object input improves compatibility without changing intrinsic signatures.

## Result

- `{\"1\":\"alice\",\"3\":true}` becomes positional params `[\"alice\", null, true]`.
- Existing non-positional object payloads keep prior behavior and do not break callers.
- Postgres and SQLite adapters now share the same positional-object interpretation model.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 positional_object_params_expand_with_null_fill`
- `cargo test -p sec4 non_numeric_object_params_fall_back_to_single_text_param`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
