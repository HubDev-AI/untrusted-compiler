# 1514 M39 Slice: Local Postgres Credential Probe Container-Port Fix

## What changed

1. Updated `infra/local-postgres/scripts/up.sh` credential verification probe:
   - changed `psql` probe port from `"${PG_PORT:-5432}"` to fixed container-side `5432`.

## Why

`PG_PORT` is a host-port mapping setting. Credential verification executes inside the `postgres` container, where Postgres listens on `5432`. Using host `PG_PORT` in-container can produce false readiness failures when host mapping is non-default.

## Validation

1. Verified script syntax and startup path still pass:
   - `infra/local-postgres/scripts/up.sh`
2. Verified deterministic mismatch path still fails with reset guidance (from prior `1512` flow).
