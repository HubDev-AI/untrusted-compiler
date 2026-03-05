# 1527 M39 Slice: Workbench sec4-lasm Service and Smoke Matrix Bootstrap

This slice turns the prompt-first workbench plan into an executable baseline.

## What changed

1. Added a new feature-rich LASM workbench service:
   - `benchmark-suite/services/sec4-lasm-workbench/sec4.toml`
   - `benchmark-suite/services/sec4-lasm-workbench/src/main.ut`
   - `benchmark-suite/services/sec4-lasm-workbench/README.md`
   - `benchmark-suite/services/sec4-lasm-workbench/smoke.sh`
2. Service behavior in this bootstrap lane:
   - auth-gated mutating routes (`auth.require(ctx.current())`),
   - real DB intrinsic execution (`db.exec`, `db.execTx`, `db.queryOne`),
   - deterministic DB-operation introspection (`DbListRecordsResponse`).
3. Added workbench smoke-matrix orchestrator:
   - `benchmark-suite/scripts/run_workbench_smoke_matrix.sh`
   - `make -C benchmark-suite workbench-smoke`
   - writes summary artifact:
     - `benchmark-suite/results/summaries/workbench-smoke-matrix.json`
4. Updated benchmark/workbench docs:
   - `benchmark-suite/README.md`
   - `docs/plans/2026-02-26-workbench-prompt-first-cross-backend-plan.md`
   - `benchmark-suite/workbench/matrix.backends.json` marks `sec4-lasm` as `implemented-alpha`.

## Why

The workbench needed an executable starting point, not only prompt/spec docs. This provides one real implementation lane (`sec4-lasm`) plus an orchestration path that reports pass/fail/skip across backend targets while other implementations are still being generated.

## Validation

1. `cargo run -q -p sec4 -- check --path benchmark-suite/services/sec4-lasm-workbench`
2. `benchmark-suite/services/sec4-lasm-workbench/smoke.sh`
3. `make -C benchmark-suite workbench-smoke`
