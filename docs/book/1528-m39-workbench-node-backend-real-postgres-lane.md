# 1528 M39 Slice: Workbench Node Backend Real Postgres Lane

This slice adds the first non-sec4 workbench backend implementation and integrates it into the workbench smoke matrix.

## What changed

1. Added new service `benchmark-suite/services/node-workbench`:
   - `server.mjs`
   - `package.json`
   - `smoke.sh`
   - `README.md`
   - `.gitignore`
2. Node workbench backend behavior:
   - real Postgres operations via `psql` CLI execution,
   - auth-gated mutating endpoints (`Authorization: Bearer token123`),
   - deterministic success/error envelopes for all routes,
   - transactional route for task + comment creation.
3. Updated workbench matrix:
   - `benchmark-suite/workbench/matrix.backends.json` marks `node` as `implemented-alpha`.
4. Workbench orchestration now exercises both implemented lanes (`sec4-lasm` + `node`) through:
   - `make -C benchmark-suite workbench-smoke`
   - output artifact `benchmark-suite/results/summaries/workbench-smoke-matrix.json`.

## Why

Cross-backend benchmarking needs at least one non-sec4 implementation running the same feature-app lane with a real DB. This makes matrix output immediately comparative while `go`/`rust` lanes are still pending.

## Validation

1. `benchmark-suite/services/node-workbench/smoke.sh`
2. `make -C benchmark-suite workbench-smoke`
