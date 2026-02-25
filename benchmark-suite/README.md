# Benchmark Suite (M10 Scaffold)

This directory is the reproducible benchmark harness scaffold for M10.

## Goal

Measure end-to-end service behavior across identical implementations:
- Untrusted<T>
- Go
- Node.js (TypeScript)
- Rust
- optional C floor reference

## Layout

- `spec/`: endpoint contract, payloads, and DB schema
  - includes canonical output artifact contract: `spec/artifact-contract-v0.1.md`
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
   - `make -C benchmark-suite bench-profile IMPL=sec4 ENDPOINT=ping`
   - `make -C benchmark-suite bench-profile IMPL=sec4 ENDPOINT=decode`
   - `make -C benchmark-suite bench-profile IMPL=sec4 ENDPOINT=users-post`
   - `make -C benchmark-suite bench-profile IMPL=sec4 ENDPOINT=users-get`
   - DB hot-path profiles (sec4/sec4-lasm services):
     - `make -C benchmark-suite bench-profile IMPL=sec4-lasm ENDPOINT=db-hot-write`
     - `make -C benchmark-suite bench-profile IMPL=sec4-lasm ENDPOINT=db-hot-write-tx`
     - `make -C benchmark-suite bench-profile IMPL=sec4-lasm ENDPOINT=db-hot-query-one`
     - `make -C benchmark-suite bench-profile IMPL=sec4-lasm ENDPOINT=db-records`
   - step-load (knee detection): `make -C benchmark-suite bench-step-profile IMPL=sec4 ENDPOINT=decode`
   - analyze step output: `make -C benchmark-suite analyze-step-profile IMPL=sec4 ENDPOINT=decode`
   - compare step analyses: `make -C benchmark-suite compare-step-matrix IMPLS=sec4,sec4-lasm,node,go,rust ENDPOINTS=decode`
5. Convert raw wrk2 output to summary JSON:
   - `make -C benchmark-suite summarize IMPL=sec4`
6. Bundle summaries + env into one report:
   - `make -C benchmark-suite report IMPL=sec4`
7. Build endpoint comparison from report bundles:
   - `make -C benchmark-suite compare`
8. Build all-endpoint comparison matrix:
   - `make -C benchmark-suite compare-matrix`
9. Analyze tail-latency and target-coverage signals:
   - `make -C benchmark-suite analyze-matrix`
10. Generate markdown benchmark report (matrix + analysis + sec4 audit):
   - `make -C benchmark-suite publish-report`
11. Run full cross-impl orchestrator:
   - `make -C benchmark-suite bench-matrix-dry`
   - `make -C benchmark-suite bench-matrix`
   - default run includes `sec4,sec4-lasm,node,go,rust`; override with `IMPLS=sec4,sec4-lasm,node,go,rust,c`
   - override endpoint set with `ENDPOINTS=ping,decode` for focused runs
   - LASM Postgres mode for matrix/full suites:
     - `make -C benchmark-suite bench-matrix BENCH_LASM_DB_ADAPTER=postgres BENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn`
     - `make -C benchmark-suite bench-full BENCH_LASM_DB_ADAPTER=postgres BENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn`
   - DB hot-path matrix convenience target (defaults: `sec4-lasm` + db endpoints):
     - `make -C benchmark-suite bench-matrix-lasm-postgres BENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn`
     - `make -C benchmark-suite bench-full-lasm-postgres BENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn`
12. Run step-load cross-impl orchestrator:
   - `make -C benchmark-suite bench-step-matrix-dry`
   - `make -C benchmark-suite bench-step-matrix`
13. Run full combined suite (fixed + step + combined publish):
   - `make -C benchmark-suite bench-full-dry`
   - `make -C benchmark-suite bench-full`
14. Run standardized Postgres comparison suite (baseline + DB hot-path snapshots):
   - `make -C benchmark-suite bench-alpha-postgres-suite-dry BENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn`
   - `make -C benchmark-suite bench-alpha-postgres-suite BENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn`
   - writes summary: `benchmark-suite/results/summaries/alpha-postgres-comparison-suite.json`
15. Run sec4 capacity probe (1M-request threshold + peak RSS):
   - `make -C benchmark-suite sec4-capacity-probe`
   - override endpoint and target requests:
     - `make -C benchmark-suite sec4-capacity-probe CAPACITY_ENDPOINT=ping CAPACITY_TARGET_REQUESTS=1000000`
16. Run sec4 LASM cluster capacity probe (1M-request threshold + peak RSS):
   - `make -C benchmark-suite lasm-cluster-capacity-probe`
   - override project and target requests:
     - `make -C benchmark-suite lasm-cluster-capacity-probe LASM_CAPACITY_PROJECT_PATH=examples/lasm-alpha-full LASM_CAPACITY_TARGET_REQUESTS=1000000`
17. Build deterministic artifact manifest:
   - `make -C benchmark-suite artifact-manifest`
18. Verify benchmark bundle completeness:
   - `make -C benchmark-suite verify-bundle IMPLS=sec4,sec4-lasm,node,go,rust ENDPOINTS=ping,decode,users-post,users-get`
   - hash checking is on by default; use `verify_benchmark_bundle.sh --skip-hash-check ...` only when intentionally bypassing manifest integrity checks
19. Validate benchmark helper scripts:
   - `make -C benchmark-suite test-scripts`
20. Validate cross-impl service contract parity:
   - `make -C benchmark-suite test-services`
21. Stop DB:
   - `make -C benchmark-suite db-down`

## Notes

- All services must implement identical endpoint behavior defined in `spec/endpoints.md`.
- Use constant-rate load for primary comparisons.
- Prefer `wrk2` for non-dry-run profile execution; `wrk` fallback is supported with explicit warning and non-constant-rate posture.
- Keep fairness controls from `docs/book/71-benchmarking-and-comparison-spec.md`.
- `services/sec4`, `services/sec4-lasm`, `services/node`, `services/go`, `services/rust`, and `services/c` include runnable baseline contract services.
- Orchestrator embeds `sec4 audit` data into `sec4-report.json` when baseline artifact is available.
- For quick local loops, override profile runtime via env vars:
  - `BENCH_THREADS`, `BENCH_CONNECTIONS`, `BENCH_DURATION`
  - `BENCH_TARGET`, or endpoint-specific `BENCH_TARGET_PING|BENCH_TARGET_DECODE|BENCH_TARGET_USERS_POST|BENCH_TARGET_USERS_GET`
  - step-load controls: `BENCH_STEP_RATES` (comma-separated), `BENCH_STEP_DURATION`
- Override orchestrator bind port with `BENCH_PORT` (default `18085`) if needed.
- Override individual smoke-script ports with `BENCH_SMOKE_PORT` when running service smoke checks directly.
- Orchestrator now runs `scripts/preflight.sh` automatically:
  - dry-run mode uses `--dry-run-only`,
  - real runs require `wrk2` and implementation toolchains to be present.
- `users-get` profile seeds one deterministic user before load and passes `BENCH_USER_ID` into `load/wrk2/get_user.lua`.
- Profile summaries now include `memory.rssKb` and `memory.sampleSource`; matrix/compare outputs propagate leader `rssKb` for throughput/latency/memory baseline visibility.
- Matrix/step orchestrators automatically pass service PID context into profile runs for RSS sampling (`ps`); standalone `run_profile.sh` can also sample RSS when `BENCH_SERVER_PID` is set.
- Matrix runs pass selected endpoint set into report bundling, so filtered runs do not accidentally include stale endpoint summaries from previous runs.
- Per-implementation report bundles now include `selectedEndpoints` metadata when matrix runs are endpoint-filtered.
- Published markdown reports include explicit endpoint scope in the header (`Endpoints in matrix (...)`) for partial-run clarity.
- Matrix comparison now scopes to selected implementations (`IMPLS`) so stale reports from other impls are excluded.
- Published markdown reports also include implementation scope (`Implementations in matrix (...)`).
- Matrix/step/full orchestrators accept LASM runtime DB mode wiring:
  - `--lasm-db-adapter records-log|sqlite|postgres`
  - `--lasm-db-postgres-dsn-file /abs/path/to/postgres.dsn` (or `SEC4_RT_LASM_DB_POSTGRES_DSN`)
- `run_alpha_postgres_comparison_suite.sh` executes two deterministic phases (baseline + db-hot) and snapshots each phase to stable artifact suffixes:
  - `*-alpha-base.*`
  - `*-alpha-db-postgres.*`
- Step-load runner writes aggregated summaries to `results/summaries/<impl>-<endpoint>-step.json`; analyzer writes `...-step-analysis.json`.
- Step comparison matrix is written to `results/summaries/step-matrix.json` by default.
- `publish_report.sh` accepts optional step matrix input and renders a `Step-Load Signals` section when provided.
- Step matrix orchestrator runs `run_step_profile` + `analyze_step_profile` per impl/endpoint and then emits one scoped `step-matrix.json`.
- Full-suite runner chains fixed-target matrix + step matrix and republishes `results/benchmark-report.md` with both standard and step-load signals.
- Full-suite runner also emits `results/artifact-manifest.json` (sha256 + size per artifact, excluding logs).
- `verify_benchmark_bundle.sh` checks required artifacts/JSON validity and verifies sha256 hashes against `artifact-manifest.json` for selected IMPLS/ENDPOINTS.
- `run_sec4_capacity_probe.sh` builds/starts sec4 benchmark service, runs one load profile, samples peak RSS, and writes `results/summaries/sec4-capacity-probe.json` with pass/fail against request threshold.
- `run_lasm_cluster_capacity_probe.sh` runs `sec4 run --backend lasm` in cluster mode with tunable scaling flags, drives `wrk`, samples peak RSS, and writes `results/summaries/sec4-lasm-cluster-capacity-probe.json`.
- `check_regression_thresholds.sh` also supports memory guard inputs (`--max-rss-kb`, baseline `baselineRssKb` + `maxRssRegressionPct`) in addition to p99/coverage thresholds.
