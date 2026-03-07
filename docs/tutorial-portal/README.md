# Untrusted<T> Tutorial Portal

This portal is the single entrypoint for learning and operating the current alpha stack.

It is organized for a new user first, then for operators who need deeper examples, DB flows, and benchmarks.

## Start by goal

| Goal | Start here |
|---|---|
| Install tools and verify local setup | `00-environment-setup.md` |
| Use Zed with plugin + language server | `01-zed-plugin-workflow.md` |
| Build and run a first server quickly | `02-build-your-first-server.md` |
| Run the canonical LASM + Postgres operator app | `03-real-db-postgres-lasm.md` |
| Explore all runnable examples | `04-example-catalog.md` |
| Run cross-backend benchmarks and capacity probes | `05-benchmarks-capacity.md` |
| Resolve common failures quickly | `06-troubleshooting.md` |

## Learning path (recommended)

1. Read `00-environment-setup.md`.
2. Complete `01-zed-plugin-workflow.md`.
3. Complete `02-build-your-first-server.md`.
4. Complete `03-real-db-postgres-lasm.md`.
5. Use `04-example-catalog.md` to pick deeper samples.
6. Use `05-benchmarks-capacity.md` for comparison and scaling runs.

## What is covered today

- Language + CLI fundamentals:
  - `sec4 init`, `sec4 check`, `sec4 build`, `sec4 run`
- IDE workflow:
  - Zed extension install/update/rollback + smoke checks
- Canonical alpha runtime path:
  - LASM + Postgres via `benchmark-suite/services/sec4-lasm-workbench`
  - canonical operator smoke contracts:
    - `benchmark-suite/services/sec4-lasm-workbench/smoke.sh`
    - `benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh`
- Additional adapter coverage (non-canonical examples):
  - `records` (`records.log`)
  - `sqlite` (`records.sqlite3`)
  - `postgres` (real Postgres DSN)
- Runtime capabilities exercised in examples:
  - HTTP routes, auth/middleware, typed request sources, typed sinks
  - DB, FS, NET intrinsics
- Benchmarks:
  - sec4 / sec4-lasm / node / go / rust comparison lanes
  - workbench benchmark matrix + report generation

## Primary references

- Core project README: `README.md`
- Long-form implementation book: `docs/book/README.md`
- Roadmap/state: `docs/05-sec4-master-roadmap.md`
- Operator handoff contract: `docs/codex-operator-handoff.md`
