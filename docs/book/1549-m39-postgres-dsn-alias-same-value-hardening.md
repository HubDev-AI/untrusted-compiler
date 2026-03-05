# M39 - Postgres DSN Alias Same-Value Hardening

## What changed

- Updated LASM Postgres DSN resolution in `compiler/sec4-cli/src/lasm_db_config.rs`:
  - `resolve_unique_env_value(...)` now accepts multiple alias env vars when values are identical.
  - `resolve_unique_env_file_path_with_candidates(...)` now accepts multiple alias file env vars when they resolve to the same normalized file path.
  - `parse_env_file_value(...)` now accepts duplicate DSN alias keys in runtime env files only when the parsed values are identical.

## Why it exists

Operators commonly set both legacy and current alias keys during env migration windows. The previous behavior rejected all duplicate alias sources, even when they carried the same value/path, causing avoidable startup failures.

## Behavior contract

- Matching duplicates are accepted.
- Conflicting duplicates still fail deterministically with the existing ambiguity diagnostics.
- Missing-file and empty-value validations remain unchanged.

## Test coverage

Added/updated focused tests in `compiler/sec4-cli/src/lasm_db_config.rs`:

- `postgres_dsn_env_aliases_with_same_value_resolve`
- `postgres_dsn_file_sources_with_same_path_resolve`
- `postgres_dsn_runtime_env_file_accepts_duplicate_aliases_when_values_match`

Existing conflict tests continue to pass and enforce deterministic failure on mismatched aliases.
