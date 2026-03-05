# 1525 M39 Slice: LASM DB Indexed Markers Require Op-Count

This slice hardens internal DB marker contract enforcement for operation-sequence dispatch.

## What changed

1. Added indexed-marker detection helpers in LASM DB runtime dispatch:
   - detect whether response headers contain indexed DB operation markers (`...-<index>` suffix) across DB internal marker families (`op`, `handle`, `template`, `params`, `tx`, `tx-db`, `row-schema`).
2. Enforced sequence-marker contract:
   - when indexed markers are present but `X-Sec4-Internal-Db-Op-Count` marker is absent, runtime now fails deterministically instead of attempting single-op fallback.
3. Deterministic failure behavior:
   - status `400`
   - code `DB.OPERATION_INVALID`
   - message `indexed internal db operation markers require operation count marker`.
4. Added focused runtime unit coverage:
   - `rejects_indexed_internal_db_markers_without_operation_count`.

## Why

Indexed DB markers represent sequence mode. Without an explicit operation-count marker, sequence metadata is malformed and should not flow through single-op parsing. This change makes that contract strict and fail-fast.

## Validation

1. `cargo test -p sec4 lasm_db_runtime_dispatch::tests::`
2. `cargo test -p sec4 --test commands run_command_lasm_backend_persists_records_log_and_query_one_when_db_intrinsics_are_used`

