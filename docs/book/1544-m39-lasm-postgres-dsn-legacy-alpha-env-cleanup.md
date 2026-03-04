# 1544 M39 Slice: Canonical LASM Postgres DSN Env Cleanup

## What changed

1. Removed legacy `SEC4_DB_ALPHA_DB_POSTGRES_DSN` from LASM Postgres DSN resolver fallback order.
2. Removed legacy `SEC4_DB_ALPHA_POSTGRES_DSN_FILE` / `SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH` from DSN file fallback.
3. Removed legacy `SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE` / `SEC4_DB_ALPHA_POSTGRES_RUNTIME_DSN_FILE` from runtime-env-file fallback.
4. Updated cluster worker forwarding to export only `SEC4_RT_LASM_DB_POSTGRES_DSN`.
5. Updated missing-DSN guidance text and command coverage assertion to canonical env-only sources.

## Why

The project is moving to `SEC4_RT_LASM_*` env names for LASM Postgres DSN resolution. Keeping legacy `SEC4_DB_ALPHA_*` DSN keys in resolver diagnostics and environment fallback weakens the migration boundary and obscures required operator inputs.

## How it works internally

`resolve_lasm_dynamic_db_postgres_dsn` now checks, in order:

1. explicit `--db-postgres-dsn` input,
2. explicit `--db-postgres-dsn-file` (handled before this function),
3. `SEC4_RT_LASM_DB_POSTGRES_DSN`,
4. `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE` / `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH`,
5. runtime env file values from `SEC4_RT_LASM_DB_POSTGRES_RUNTIME_ENV_FILE` / `SEC4_RT_LASM_DB_POSTGRES_RUNTIME_DSN_FILE`.

If no canonical source exists, the function returns `LASM_DB_POSTGRES_DSN_CONFIG_ERROR_MESSAGE`.

## Inputs/outputs and constraints

Inputs:

- environment variables listed in the canonical LASM DSN names above,
- project path for relative file resolution,
- explicit CLI DSN sources passed through `main`/`cmd_run`.

Outputs:

- `Some(dsn)` when one canonical source resolves,
- `Err(...)` with a deterministic message when no source is available.

Constraint: legacy `SEC4_DB_ALPHA_*` DSN variables are intentionally ignored by the resolver.

## Failure modes and diagnostics

- missing DSN: `run failed: db adapter postgres requires --db-postgres-dsn or a DSN source via ...`
- empty DSN file content: explicit non-empty checks still reject blank/whitespace-only values,
- unreadable DSN/runtime-env files: existing file-path diagnostics remain unchanged.

## Example usage

- canonical Postgres DSN env: `SEC4_RT_LASM_DB_POSTGRES_DSN=postgres://...`
- canonical DSN file env: `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE=/path/to/dsn`

## Tradeoffs and next steps

Tradeoff: dropping legacy DSN env names is intentionally a stricter operator contract. Compatibility is preserved where needed by using explicit `--db-postgres-dsn` and existing wrapper flags.

Next steps:

- update out-of-scope operator documentation/scripts that still advertise only legacy `SEC4_DB_ALPHA_*` DSN env keys, if migration support is no longer desired there.
