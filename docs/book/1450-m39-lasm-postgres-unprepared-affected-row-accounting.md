# M39: LASM Postgres Unprepared Affected-Row Accounting

Date: 2026-02-22  
Milestone: M39 (Postgres runtime execution fidelity)

## What Changed

- Updated Postgres runtime execution paths for unprepared SQL (`no params`):
  - added unprepared execution helper that:
    - uses direct `execute` count for single-statement commands,
    - falls back to query-row counting when execution reports row-returning behavior,
    - keeps batch-execute (`0`) behavior for multi-statement SQL.
- Wired helper into both:
  - `run_lasm_postgres_exec`,
  - `run_lasm_postgres_exec_tx_once`.

## Why

Unprepared execution previously used batch-execute-only behavior and often returned `0` affected rows even for statements where deterministic row/affected counts were available.

## Result

- Non-parameterized single statements now provide better affected-row fidelity.
- Multi-statement behavior remains deterministic and unchanged.
- Retry and error-mapping flow stays aligned with existing runtime contracts.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_lasm_backend_exec_and_query_one_with_postgres_adapter_when_dsn_available`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
