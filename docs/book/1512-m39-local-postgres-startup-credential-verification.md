# 1512 M39 Slice: Local Postgres Startup Credential Verification

## What changed

1. Hardened `infra/local-postgres/scripts/up.sh` with a real credentialed readiness check:
   - after Docker health reports running/healthy, the script now executes:
     - `psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "select 1"` (with `ON_ERROR_STOP=1`),
   - probe uses configured local env (`POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB`, `PG_PORT`).
2. Added deterministic mismatch guidance:
   - if credential verification fails, startup now exits immediately with:
     - clear mismatch context (`POSTGRES_USER`, `POSTGRES_DB`),
     - explicit recovery instruction to run `infra/local-postgres/scripts/reset.sh`.

## Why

Docker health-only readiness is not enough when persisted `infra/local-postgres/data` was initialized with different credentials in an earlier run. In that case, benchmark/operator flows could reach LASM runtime startup and fail later with opaque DB connect errors. This change fails fast in infra startup and gives one deterministic recovery path.

## Validation

1. Positive path:
   - `infra/local-postgres/scripts/down.sh`
   - `infra/local-postgres/scripts/up.sh`
   - confirmed successful startup and DSN hints.
2. Negative path (credential mismatch against existing data):
   - replace `infra/local-postgres/.env` with mismatched user/password,
   - `infra/local-postgres/scripts/down.sh`
   - `infra/local-postgres/scripts/up.sh`
   - confirmed deterministic failure and reset guidance:
     - `configured Postgres credentials failed verification ...`
     - `hint: ... run .../infra/local-postgres/scripts/reset.sh`
