# 1544 M39 Slice: LASM Postgres DSN Env Cleanup (Superseded)

This slice was initially committed as a strict migration to canonical `SEC4_RT_LASM_*` DSN names.
The project later widened compatibility to accept both DSN families again (`SEC4_DB_ALPHA_*` and
`SEC4_RT_LASM_*`) with deterministic guidance, so this record is now historical only.

## Historical behavior (superseded)

This work originally:

1. Preferred canonical `SEC4_RT_LASM_*` env names for LASM Postgres DSN resolution.
2. Removed `SEC4_DB_ALPHA_*` fallbacks from resolver order.
3. Standardized diagnostics around canonical env-only guidance.

It was superseded by compatibility restoration for `SEC4_DB_ALPHA_*` aliases.

## Current contract

`resolve_lasm_dynamic_db_postgres_dsn` and wrapper scripts now accept both alias families
for DSN-source compatibility:

- `SEC4_DB_ALPHA_DB_POSTGRES_DSN` or `SEC4_RT_LASM_DB_POSTGRES_DSN`
- `SEC4_DB_ALPHA_POSTGRES_DSN_FILE` / `SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH` or
  `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE` / `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH`

Cluster forwarding and user-facing guidance were updated accordingly.
