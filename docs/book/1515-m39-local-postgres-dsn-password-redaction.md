# 1515 M39 Slice: Local Postgres DSN Password Redaction

## What changed

1. Updated `infra/local-postgres/scripts/up.sh` startup output:
   - added `redact_postgres_dsn_password` helper,
   - DSN hint output now redacts password segment:
     - `postgres://user:***@host:port/db?...`

## Why

Local infra startup printed full DSN including raw password. Even in local tooling flows, this leaks credentials into terminal history/log captures. Redacted DSN keeps operator usability while reducing credential exposure.

## Validation

1. Ran:
   - `infra/local-postgres/scripts/up.sh`
2. Confirmed startup output shows redacted DSN password (`***`) instead of raw `POSTGRES_PASSWORD`.
