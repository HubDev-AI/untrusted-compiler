# Postgres E2E Durable Setup Design

Date: 2026-02-22  
Status: Approved
Owner: Codex + Operator
Milestone: M39 (LASM backend + real DB runtime proof)

## Goal
Provide a durable, repeatable, real Postgres end-to-end backend flow in-repo so operators can run LASM against a real external DB (not stubs) and validate `db.exec`, `db.execTx`, `db.queryOne`, and `DbListRecordsResponse` telemetry.

## Chosen Approach (Hybrid)
Use both:
1. Shared local infra in `infra/local-postgres/`
2. Dedicated runnable example in `examples/postgres-e2e/`

This keeps infrastructure reusable while also providing a single clear demo path.

## Architecture

### Shared Infra
Create:
- `infra/local-postgres/docker-compose.yml`
- `infra/local-postgres/.env.example`
- `infra/local-postgres/scripts/up.sh`
- `infra/local-postgres/scripts/down.sh`
- `infra/local-postgres/scripts/reset.sh`

Responsibilities:
- Start/stop/reset local Postgres deterministically
- Publish DSN/env contract for local runs
- Serve as common infra for current and future examples

### Example App
Create:
- `examples/postgres-e2e/sec4.toml`
- `examples/postgres-e2e/src/main.ut`
- `examples/postgres-e2e/README.md`
- optional: `examples/postgres-e2e/scripts/smoke.sh`

Responsibilities:
- Demonstrate real runtime DB intrinsics through HTTP routes
- Be runnable without hidden context from repo root docs
- Show operator-visible results and diagnostics

## Runtime/Data Flow
1. Bring up Postgres with compose from `infra/local-postgres/`.
2. Export runtime env for LASM process:
  - `SEC4_DB_ALPHA_DB_POSTGRES_DSN=<infra dsn>` (preferred) or
    `SEC4_RT_LASM_DB_POSTGRES_DSN=<infra dsn>`
3. Run example with LASM backend:
   - `sec4 run --backend lasm --path examples/postgres-e2e --oneshot --port <port>`
4. Exercise routes:
   - `/db/exec`
   - `/db/exec-tx`
   - `/db/query-one`
   - `/db/records`

Expected result:
- Real Postgres-backed operations complete
- Response payloads include DB-driven values
- `/db/records` includes `"adapter":"postgres"` and runtime telemetry fields

## Error Handling and Operator UX
Infra scripts must fail fast and print actionable next commands for:
- Docker unavailable
- Docker daemon down
- Port collisions
- Missing env values

Example README must include deterministic failure diagnostics for:
- missing DSN
- Postgres unreachable
- authentication failure

## Verification Scope (Results-First)
Minimum required proof:
1. Start Postgres infra
2. Run LASM backend with Postgres adapter
3. Call `exec`, `execTx`, `queryOne`, and `records` routes via `curl`
4. Confirm expected HTTP statuses and telemetry fields

Validation policy for this slice:
- Focused verification only for touched flow
- Avoid broad unrelated test sweeps

## Implementation Order
1. Add `infra/local-postgres` (compose + env contract + scripts)
2. Add `examples/postgres-e2e` app + README + smoke script
3. Add one focused integration validation (commands-level)
4. Update roadmap + docs/book chapter + index

## Constraints
- No language semantic changes in this slice
- No stubs/mocks for DB path
- Keep LASM as runtime target for this proof
- Keep changes scoped and operator-friendly

## Acceptance Criteria
- Operator can run one documented flow from repo and hit real Postgres via LASM routes
- Route responses show real DB behavior (not placeholders)
- `/db/records` confirms adapter + telemetry
- Documentation is sufficient for another engineer to reproduce without chat context

## Tradeoffs
- Hybrid adds slightly more initial setup than single-folder demo
- In return it prevents infra duplication and scales to more examples

## Next
After this lands: continue M39 performance tuning and keep DB runtime improvements measurable via the same example/infra harness.
