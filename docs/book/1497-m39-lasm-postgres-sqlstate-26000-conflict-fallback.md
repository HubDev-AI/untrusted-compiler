# M39 Slice: LASM Postgres SQLSTATE 26000 Conflict Fallback

Date: 2026-02-26  
Milestone: M39 (LASM DB client completion lane)

## What changed

- Updated `classify_lasm_db_runtime_error` to map Postgres `sqlstate=26000` to deterministic conflict envelopes (`409`, `DB.*_CONFLICT`, `kind=conflict`).
- Added focused runtime classifier coverage for a bare `26000` failure string without stale-plan text markers.

## Why

- Postgres stale prepared statement failures can surface through different message variants across adapters/driver contexts.
- Some variants provide only SQLSTATE metadata without the stable stale-plan keywords.
- Mapping `26000` directly prevents drift back to non-conflict classification for unrecovered stale-plan failures.

## Behavioral contract

- For `exec`, `execTx`, and `queryOne`, errors carrying `sqlstate=26000` classify as conflict regardless of message text details.
- Existing stale-plan keyword mapping remains in place.
- Validation/unavailable/internal mappings for other SQLSTATE classes are unchanged.

## Validation

- `cargo test -p sec4 classify_db_runtime_sqlstate_invalid_statement_name_as_conflict`
- `cargo test -p sec4 classify_db_runtime_sqlstate_feature_not_supported_as_validation`
