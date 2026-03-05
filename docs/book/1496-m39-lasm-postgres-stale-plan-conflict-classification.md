# M39 Slice: LASM Postgres Stale-Plan Conflict Classification

Date: 2026-02-26  
Milestone: M39 (LASM DB client completion lane)

## What changed

- Added shared DB runtime stale-plan classifier in `classify_lasm_db_runtime_error` for Postgres stale prepared statement signatures:
  - `prepared statement ... does not exist`
  - `cached plan must not change result type`
- Updated deterministic runtime error envelope mapping so stale-plan leftovers return conflict envelopes (`409`, `DB.*_CONFLICT`, `kind=conflict`) instead of validation envelopes.
- Kept generic `sqlstate=0A000` (`feature not supported`) mapped to validation when stale-plan signatures are not present.

## Why

- Prepared execution already attempts stale-plan eviction/reprepare, but unrecovered stale-plan failures could still bubble as validation errors.
- Stale-plan residue is runtime cache drift, not user input shape failure; conflict classification is a better deterministic contract for operators and callers.

## Behavioral contract

- Any DB runtime error containing stale-plan signature text is classified as conflict for `exec`, `execTx`, and `queryOne`.
- Existing validation classification for non-stale `feature not supported` errors remains unchanged.
- Adapter config/internal/unavailable/timeout classification behavior remains unchanged.

## Validation

- `cargo test -p sec4 classify_db_runtime_stale_prepared_statement_error_as_conflict`
- `cargo test -p sec4 classify_db_runtime_cached_plan_shape_drift_as_conflict`
- `cargo test -p sec4 classify_db_runtime_sqlstate_feature_not_supported_as_validation`
