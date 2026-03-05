# M39: Postgres E2E Local Infra Example and Smoke Flow

Date: 2026-02-22  
Milestone: M39 (alpha usability + DB operator flow)

## What Changed

- Added shared local Postgres infra under `infra/local-postgres`:
  - `docker-compose.yml`
  - `.env.example`
  - `scripts/up.sh`
  - `scripts/down.sh`
  - `scripts/reset.sh`
- Added canonical real-DB example under `examples/postgres-e2e`:
  - `sec4.toml`
  - `sec4.policy`
  - `src/main.ut`
  - `scripts/smoke.sh`
  - `README.md`
- Added focused CLI integration check in `compiler/sec4-cli/tests/commands.rs`:
  - `check_command_succeeds_for_postgres_e2e_example`

## Why

The alpha DB path needed one deterministic operator flow that starts a real Postgres instance, runs LASM routes using `db.exec`/`db.execTx`/`db.queryOne`, and verifies runtime DB telemetry output without requiring ad-hoc setup.

## Result

- Operators can start local Postgres with one command and run a reproducible smoke flow against real DB execution.
- The example demonstrates end-to-end intrinsic usage and runtime telemetry exposure (`adapter`, statement/placeholder cache fields).
- CI-facing command coverage now locks that the example compiles through `sec4 check`.

## Validation

- `cargo run -p sec4 -- check --path $REPO_ROOT/examples/postgres-e2e`
- `bash -n $REPO_ROOT/infra/local-postgres/scripts/up.sh`
- `bash -n $REPO_ROOT/infra/local-postgres/scripts/down.sh`
- `bash -n $REPO_ROOT/infra/local-postgres/scripts/reset.sh`
- `bash -n $REPO_ROOT/examples/postgres-e2e/scripts/smoke.sh`
- `cargo test -p sec4 --test commands check_command_succeeds_for_postgres_e2e_example`
