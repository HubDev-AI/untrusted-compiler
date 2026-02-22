# M39: LASM DB Cache Telemetry in DbListRecordsResponse

Date: 2026-02-22  
Milestone: M39 (DB runtime hardening)

## What Changed

- Extended `DbListRecordsResponse` materialization in `compiler/sec4-cli/src/main.rs` with `dbCache` telemetry:
  - `postgresStatementCount`
  - `postgresStatementCapacity`
  - `postgresPlaceholderCount`
  - `postgresPlaceholderCapacity`
- Updated records-log DB command integration assertion in `compiler/sec4-cli/tests/commands.rs` to verify deterministic presence of the new `dbCache` fields.

## Why

After adding runtime statement/placeholder caches and capacity guards, operators needed an easy way to confirm live cache state without attaching debuggers or reading internal logs.

## Result

- `GET /db/records` now exposes runtime DB cache utilization and configured capacities in-band with existing DB runtime telemetry.
- Works across DB adapters because cache telemetry comes from shared dynamic state.
- Existing response semantics (`count`, `affectedRowsTotal`, adapter/tx telemetry, records list) remain unchanged.

## Validation

- `rustfmt compiler/sec4-cli/src/main.rs compiler/sec4-cli/tests/commands.rs`
- `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`
