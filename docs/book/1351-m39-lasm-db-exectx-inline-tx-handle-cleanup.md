# M39: LASM DB execTx Inline Tx-Handle Cleanup

Date: 2026-02-22  
Milestone: M39-S2B (LASM runtime + DB hardening)

## What Changed

- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`:
  - `execTx` now removes auto-allocated inline tx handles (`db.execTx(db.tx(dbCap), query)` materialization path) from runtime tx-handle registry after successful DB execution.
- Updated `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/tests/commands.rs`:
  - adjusted sqlite tx-capacity command integration expectations to assert successful second inline `execTx` request and bounded tx-handle telemetry (`txHandleCount=0`, `txHandleCapacity=1`).

## Why

Inline `db.tx(dbCap)` allocation is a one-shot execution path. Keeping those handles after successful `execTx` leaked tx-handle capacity and could trigger deterministic `DB.TX_INTERNAL` failures under sustained traffic without providing persistent transaction value.

## Result

- No capacity leak from successful inline `execTx` calls.
- Existing invalid/stale tx-handle guards remain unchanged.
- DB record history still captures deterministic `execTx` records and affected-row metadata.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_backend_invalid_exec_tx_handle_does_not_execute_sql_when_sqlite_adapter`
- `cargo test -p sec4 --test commands run_command_lasm_backend_rejects_stale_exec_tx_handle_after_restart_when_sqlite_adapter`
- `cargo test -p sec4 --test commands run_command_lasm_backend_enforces_db_tx_handle_capacity_with_cli_flag_when_sqlite_adapter`
