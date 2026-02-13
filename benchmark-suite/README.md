# Benchmark Suite (M10 Scaffold)

This directory is the reproducible benchmark harness scaffold for M10.

## Goal

Measure end-to-end service behavior across identical implementations:
- AILang
- Go
- Node.js (TypeScript)
- Rust
- optional C floor reference

## Layout

- `spec/`: endpoint contract, payloads, and DB schema
- `services/`: per-language service implementations (to be added)
- `load/wrk2/`: load scripts
- `results/`: raw outputs, summaries, plots
- `docker-compose.yml`: shared Postgres for DB endpoints
- `Makefile`: benchmark orchestration entrypoints

## Initial workflow

1. Start DB:
   - `make -C benchmark-suite db-up`
2. Apply schema:
   - `make -C benchmark-suite db-schema`
3. Validate benchmark tooling before runs:
   - `make -C benchmark-suite preflight`
   - for dry-run-only checks: `make -C benchmark-suite preflight-dry`
4. Run benchmark profile (once service is implemented):
   - `make -C benchmark-suite bench-profile IMPL=ailang ENDPOINT=ping`
   - `make -C benchmark-suite bench-profile IMPL=ailang ENDPOINT=decode`
   - `make -C benchmark-suite bench-profile IMPL=ailang ENDPOINT=users-post`
   - `make -C benchmark-suite bench-profile IMPL=ailang ENDPOINT=users-get`
5. Convert raw wrk2 output to summary JSON:
   - `make -C benchmark-suite summarize IMPL=ailang`
6. Bundle summaries + env into one report:
   - `make -C benchmark-suite report IMPL=ailang`
7. Build endpoint comparison from report bundles:
   - `make -C benchmark-suite compare`
8. Build all-endpoint comparison matrix:
   - `make -C benchmark-suite compare-matrix`
9. Analyze tail-latency and target-coverage signals:
   - `make -C benchmark-suite analyze-matrix`
10. Generate markdown benchmark report (matrix + analysis + sec.audit):
   - `make -C benchmark-suite publish-report`
11. Run full cross-impl orchestrator:
   - `make -C benchmark-suite bench-matrix-dry`
   - `make -C benchmark-suite bench-matrix`
   - default run includes `ailang,node,go,rust`; override with `IMPLS=ailang,node,go,rust,c`
   - override endpoint set with `ENDPOINTS=ping,decode` for focused runs
12. Validate benchmark helper scripts:
   - `make -C benchmark-suite test-scripts`
13. Validate cross-impl service contract parity:
   - `make -C benchmark-suite test-services`
14. Stop DB:
   - `make -C benchmark-suite db-down`

## Notes

- All services must implement identical endpoint behavior defined in `spec/endpoints.md`.
- Use constant-rate load for primary comparisons.
- Ensure `wrk2` is installed and available in `PATH` for non-dry-run profile execution.
- Keep fairness controls from `docs/book/71-benchmarking-and-comparison-spec.md`.
- `services/ailang`, `services/node`, `services/go`, `services/rust`, and `services/c` include runnable baseline contract services.
- Orchestrator embeds `sec.audit` data into `ailang-report.json` when baseline artifact is available.
- For quick local loops, override profile runtime via env vars:
  - `BENCH_THREADS`, `BENCH_CONNECTIONS`, `BENCH_DURATION`
  - `BENCH_TARGET`, or endpoint-specific `BENCH_TARGET_PING|BENCH_TARGET_DECODE|BENCH_TARGET_USERS_POST|BENCH_TARGET_USERS_GET`
- Override orchestrator bind port with `BENCH_PORT` (default `18085`) if needed.
- Override individual smoke-script ports with `BENCH_SMOKE_PORT` when running service smoke checks directly.
- Orchestrator now runs `scripts/preflight.sh` automatically:
  - dry-run mode uses `--dry-run-only`,
  - real runs require `wrk2` and implementation toolchains to be present.
- `users-get` profile seeds one deterministic user before load and passes `BENCH_USER_ID` into `load/wrk2/get_user.lua`.
