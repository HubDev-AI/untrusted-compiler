# 1526 M39 Slice: LASM DB Required Template Marker Hardening

This slice removes a remaining compatibility fallback in LASM DB internal operation dispatch.

## What changed

1. Required explicit internal SQL template marker headers for DB operations:
   - `exec`
   - `execTx`
   - `queryOne`
2. Removed `unwrap_or_default()` fallback for missing template headers in those paths.
3. Missing template markers now fail deterministically with operation-specific validation envelopes:
   - `DB.EXEC_INVALID`
   - `DB.EXEC_TX_INVALID`
   - `DB.QUERY_ONE_INVALID`
4. Added focused runtime unit coverage:
   - `exec_marker_rejects_missing_template_header`
   - `exec_tx_marker_rejects_missing_template_header`
   - `query_one_marker_rejects_missing_template_header`

## Why

Missing template markers indicate malformed internal DB operation metadata and should fail fast with deterministic operation-specific diagnostics. Silent defaulting to empty template values adds ambiguity and weakens strict no-stub runtime contracts.

## Validation

1. `cargo test -p sec4 lasm_db_runtime_dispatch::tests::`
