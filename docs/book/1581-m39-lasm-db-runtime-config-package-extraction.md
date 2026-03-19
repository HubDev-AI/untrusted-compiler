# 1581 M39: LASM DB Runtime Config Package Extraction

## What it is

This slice extracts reusable LASM DB runtime configuration logic from `sec4-cli` into a dedicated workspace crate:

- `compiler/sec4-lasm-db-client`

The new crate now owns:

- DB runtime limit resolution from env (sql template bytes, params bytes/entries, op sequence max, queryOne row bytes/columns),
- runtime limit override application (`LasmDbRuntimeLimitOverrides`),
- DB records capture/persist env gating resolution.

`sec4-cli` keeps CLI/runtime-state-bound logic only:

- Postgres thread-local config build/cache in `compiler/sec4-cli/src/lasm_db_client/config.rs`.

## Why it exists

Before this extraction, reusable runtime config logic was mixed with CLI dynamic-state code under `compiler/sec4-cli/src/lasm_db_client/config.rs`.
That made packaging boundaries blurry and kept shared DB client behavior coupled to the CLI binary crate.

The new package boundary reduces coupling and makes future DB-client extraction steps incremental instead of all-or-nothing.

## How it works internally

1. Added workspace crate:
   - `compiler/sec4-lasm-db-client/Cargo.toml`
   - `compiler/sec4-lasm-db-client/src/lib.rs`
2. Moved reusable runtime config surfaces there:
   - `resolve_lasm_db_*` functions
   - `set_lasm_db_*_override` functions
   - `apply_lasm_db_runtime_limit_overrides`
   - `lasm_db_records_persist_enabled`
   - `lasm_db_records_capture_enabled`
3. Rewired `sec4-cli` call sites:
   - `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs` now imports limit/override APIs from `sec4_lasm_db_client`.
   - `compiler/sec4-cli/src/lasm_db_client/adapter.rs` delegates records capture/persist env checks to `sec4_lasm_db_client`.
4. Reduced `sec4-cli` local config module to Postgres state-bound config builder only.

## Inputs/outputs and constraints

- Inputs:
  - runtime env variables (`SEC4_RT_LASM_DB_*`, `SEC4_DB_ALPHA_DB_RECORDS_*` aliases),
  - CLI-applied runtime overrides.
- Outputs:
  - resolved numeric limits,
  - resolved capture/persist feature flags,
  - same behavior for LASM DB dispatch/runtime limits as before extraction.
- Constraints:
  - no language-semantics changes,
  - no CLI UX contract changes,
  - keep deterministic clamping/default behavior.

## Failure modes and diagnostics

- Invalid env values continue to fall back deterministically to defaults/clamped bounds.
- Missing env values continue to use defaults.
- Extraction is compile-time validated through workspace dependency wiring.

## Example usage

`sec4-cli` runtime dispatch now consumes the package directly:

- `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`
  - `resolve_lasm_db_sql_template_max_bytes`
  - `resolve_lasm_db_params_max_bytes`
  - `resolve_lasm_db_params_max_entries`
  - `resolve_lasm_db_op_sequence_max`
  - `resolve_lasm_db_query_one_row_max_bytes`
  - `resolve_lasm_db_query_one_row_max_columns`
  - `apply_lasm_db_runtime_limit_overrides`

## Tradeoffs and next steps

- Tradeoff:
  - one more workspace crate and dependency edge,
  - but cleaner package boundary and lower coupling in `sec4-cli`.
- Next steps:
  - continue extracting DB client adapter/runtime logic into package-friendly layers,
  - keep CLI state-bound code local where direct access to `LasmDynamicResponseState` is required.
