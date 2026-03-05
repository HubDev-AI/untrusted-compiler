# 1524 M39 Slice: LASM DB Empty Params Marker Payload Rejection

This slice hardens LASM internal DB marker handling by rejecting empty internal SQL params payloads.

## What changed

1. Added strict params-payload validation helper in LASM DB runtime dispatch:
   - `enforce_lasm_db_params_required(...)` now rejects empty/whitespace params payloads.
2. Applied params-payload validation across DB runtime operation paths:
   - `exec`
   - `execTx`
   - `queryOne`
3. Deterministic failure behavior:
   - operation-specific validation code:
     - `DB.EXEC_INVALID`
     - `DB.EXEC_TX_INVALID`
     - `DB.QUERY_ONE_INVALID`
   - message: `sql.q params payload is required`
   - status: `400`.
4. Added focused runtime unit coverage:
   - `exec_marker_rejects_empty_params_header`
   - `query_one_marker_rejects_empty_params_header`

## Why

Empty internal params payloads were previously normalized into implicit no-params behavior, which could hide malformed internal marker state. This change keeps marker contracts explicit and fail-fast.

## Validation

1. `cargo test -p sec4 lasm_db_runtime_dispatch::tests::`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`

