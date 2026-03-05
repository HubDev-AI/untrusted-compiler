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
- `workbench/`: prompt-first feature-app benchmark scaffold (contract + generation prompts + backend matrix)
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
   - repeated same-condition run-manifest wrapper:
     - `make -C benchmark-suite bench-alpha-postgres-suite-repeats-dry BENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn BENCH_ALPHA_POSTGRES_REPEAT_RUNS=3`
     - `make -C benchmark-suite bench-alpha-postgres-suite-repeats BENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn BENCH_ALPHA_POSTGRES_REPEAT_RUNS=3`
   - optional DB cleanup between baseline/db-hot phases:
     - `benchmark-suite/scripts/run_alpha_postgres_comparison_suite.sh --reset-db-between-phases ...`
     - or from Make targets: `BENCH_ALPHA_POSTGRES_RESET_BETWEEN_PHASES=true`
   - writes summary: `benchmark-suite/results/summaries/alpha-postgres-comparison-suite.json`
   - repeated wrapper writes aggregate summary: `benchmark-suite/results/summaries/alpha-postgres-comparison-suite-repeats.json`
15. Run standardized Postgres comparison suite with repo-local Postgres infra orchestration:
   - `make -C benchmark-suite bench-alpha-postgres-suite-local-dry`
   - `make -C benchmark-suite bench-alpha-postgres-suite-local`
   - repeated local-infra run wrapper:
     - `make -C benchmark-suite bench-alpha-postgres-suite-local-repeats-dry BENCH_ALPHA_POSTGRES_REPEAT_RUNS=3`
     - `make -C benchmark-suite bench-alpha-postgres-suite-local-repeats BENCH_ALPHA_POSTGRES_REPEAT_RUNS=3`
   - render markdown from repeated summary:
     - `make -C benchmark-suite bench-alpha-postgres-suite-repeats-report`
   - optional:
     - keep infra running after suite: `BENCH_LOCAL_POSTGRES_KEEP_UP=true`
     - recreate local postgres data before suite: `BENCH_LOCAL_POSTGRES_RESET=true`
16. Run sec4 capacity probe (1M-request threshold + peak RSS):
   - `make -C benchmark-suite sec4-capacity-probe`
   - override endpoint and target requests:
     - `make -C benchmark-suite sec4-capacity-probe CAPACITY_ENDPOINT=ping CAPACITY_TARGET_REQUESTS=1000000`
17. Run sec4 LASM cluster capacity probe (1M-request threshold + peak RSS):
   - `make -C benchmark-suite lasm-cluster-capacity-probe`
   - override project and target requests:
     - `make -C benchmark-suite lasm-cluster-capacity-probe LASM_CAPACITY_PROJECT_PATH=examples/lasm-alpha-full LASM_CAPACITY_TARGET_REQUESTS=1000000`
18. Build deterministic artifact manifest:
   - `make -C benchmark-suite artifact-manifest`
19. Verify benchmark bundle completeness:
   - `make -C benchmark-suite verify-bundle IMPLS=sec4,sec4-lasm,node,go,rust ENDPOINTS=ping,decode,users-post,users-get`
   - hash checking is on by default; use `verify_benchmark_bundle.sh --skip-hash-check ...` only when intentionally bypassing manifest integrity checks
20. Validate benchmark helper scripts:
   - `make -C benchmark-suite test-scripts`
21. Validate cross-impl service contract parity:
   - `make -C benchmark-suite test-services`
22. Run workbench smoke matrix (prompt-first generated feature app lane):
   - `make -C benchmark-suite workbench-smoke`
23. Run workbench benchmark matrix (feature-app load profiles + compare artifacts):
   - dry-run plan:
     - `make -C benchmark-suite workbench-bench-dry`
   - full run:
     - `make -C benchmark-suite workbench-bench`
   - local-infra run:
     - `make -C benchmark-suite workbench-bench-local-dry`
     - `make -C benchmark-suite workbench-bench-local`
   - optional LASM DB mode knobs:
     - `WORKBENCH_LASM_DB_ADAPTER=sqlite|postgres`
     - `WORKBENCH_LASM_DB_BASE=/tmp/sec4-lasm-workbench-db` (sqlite mode only)
     - `WORKBENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn` (postgres mode only)
   - outputs:
     - `results/summaries/workbench-benchmark-runs.json`
     - `results/summaries/workbench-benchmark-compare-matrix.json`
     - `results/summaries/workbench-benchmark-analysis.json`
     - `results/workbench-benchmark-report.md`
     - `results/workbench-benchmark-report.html`
24. Run workbench step-load benchmark matrix (feature-app knee detection + step matrix):
   - dry-run plan:
     - `make -C benchmark-suite workbench-step-bench-dry`
   - full run:
     - `make -C benchmark-suite workbench-step-bench`
   - local-infra run:
     - `make -C benchmark-suite workbench-step-bench-local-dry`
     - `make -C benchmark-suite workbench-step-bench-local`
   - optional scope overrides:
     - `WORKBENCH_IMPLS=sec4,sec4-lasm`
     - `WORKBENCH_ENDPOINTS=wb-task-get,wb-tasks-list`
   - optional LASM DB mode knobs:
     - `WORKBENCH_LASM_DB_ADAPTER=sqlite|postgres`
     - `WORKBENCH_LASM_DB_BASE=/tmp/sec4-lasm-workbench-db` (sqlite mode only)
     - `WORKBENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn` (postgres mode only)
   - outputs:
     - `results/summaries/workbench-step-runs.json`
     - `results/summaries/workbench-step-matrix.json`
     - per-step summaries and analyses:
       - `results/summaries/<impl>-<endpoint>-step.json`
       - `results/summaries/<impl>-<endpoint>-step-analysis.json`
25. Run workbench full benchmark suite (fixed-target + step-load + combined report):
   - dry-run plan:
     - `make -C benchmark-suite workbench-full-bench-dry`
   - full run:
     - `make -C benchmark-suite workbench-full-bench`
   - optional scope overrides:
     - `WORKBENCH_IMPLS=sec4,sec4-lasm`
     - `WORKBENCH_ENDPOINTS=wb-task-get,wb-tasks-list`
   - optional LASM DB mode knobs:
     - `WORKBENCH_LASM_DB_ADAPTER=sqlite|postgres`
     - `WORKBENCH_LASM_DB_BASE=/tmp/sec4-lasm-workbench-db` (sqlite mode only)
     - `WORKBENCH_LASM_DB_POSTGRES_DSN_FILE=/abs/path/to/postgres.dsn` (postgres mode only)
   - outputs:
     - `results/summaries/workbench-full-runs.json`
     - `results/workbench-full-benchmark-report.md`
     - `results/workbench-full-benchmark-report.html`
26. Run repeated workbench full suites under identical settings:
   - dry-run plan:
     - `make -C benchmark-suite workbench-full-bench-repeats-dry`
   - full run:
     - `make -C benchmark-suite workbench-full-bench-repeats`
   - render markdown summary:
     - `make -C benchmark-suite workbench-full-bench-repeats-report`
   - repeat count override:
     - `WORKBENCH_REPEAT_RUNS=3` (default)
   - outputs:
     - `results/summaries/workbench-full-benchmark-repeats.json`
     - `results/workbench-full-benchmark-repeats.md`
     - run-scoped artifacts under:
       - `results/summaries/workbench-full-benchmark-runs/`
27. Run workbench full suite with repo-local Postgres infra orchestration:
   - dry-run plan:
     - `make -C benchmark-suite workbench-full-bench-local-dry`
   - full run:
     - `make -C benchmark-suite workbench-full-bench-local`
   - optional:
     - keep infra running after suite: `BENCH_LOCAL_POSTGRES_KEEP_UP=true`
     - recreate local postgres data before suite: `BENCH_LOCAL_POSTGRES_RESET=true`
28. Run repeated workbench full suite with repo-local Postgres infra orchestration:
   - dry-run plan:
     - `make -C benchmark-suite workbench-full-bench-local-repeats-dry WORKBENCH_REPEAT_RUNS=3`
   - full run:
     - `make -C benchmark-suite workbench-full-bench-local-repeats WORKBENCH_REPEAT_RUNS=3`
   - one-command repeated-run + markdown render bundle:
     - `make -C benchmark-suite workbench-full-bench-local-bundle-dry WORKBENCH_REPEAT_RUNS=3`
     - `make -C benchmark-suite workbench-full-bench-local-bundle WORKBENCH_REPEAT_RUNS=3`
   - outputs are the same as non-local repeated wrapper:
     - `results/summaries/workbench-full-benchmark-repeats.json`
     - `results/workbench-full-benchmark-repeats.md`
     - `results/summaries/workbench-full-benchmark-runs/`
29. Stop DB:
   - `make -C benchmark-suite db-down`
30. Re-render compact HTML report from existing JSON artifacts (no rerun required):
   - `make -C benchmark-suite workbench-bench-report-html`

## Notes

- All services must implement identical endpoint behavior defined in `spec/endpoints.md`.
- Use constant-rate load for primary comparisons.
- Workbench matrix/step/full suite runners enforce `wrk2` by default (`BENCH_WORKBENCH_REQUIRE_WRK2=1`) so compare artifacts stay constant-rate.
- Set `BENCH_WORKBENCH_REQUIRE_WRK2=0` only when you intentionally allow local `wrk` fallback (non-constant-rate posture).
- Install repo-local `wrk2` when system `wrk2` is missing:
  - `make -C benchmark-suite wrk2-install`
  - installer mode defaults to `auto` (source build, then docker-wrapper fallback if build fails)
  - then export `BENCH_WRK2_BIN="$PWD/benchmark-suite/bin/wrk2"` for benchmark runs.
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
  - `--lasm-db-postgres-dsn-file /abs/path/to/postgres.dsn` (or `SEC4_DB_ALPHA_DB_POSTGRES_DSN`; legacy alias `SEC4_RT_LASM_DB_POSTGRES_DSN`)
- `run_alpha_postgres_comparison_suite.sh` executes two deterministic phases (baseline + db-hot) and snapshots each phase to stable artifact suffixes:
  - `*-alpha-base.*`
  - `*-alpha-db-postgres.*`
  - summary (`results/summaries/alpha-postgres-comparison-suite.json`) now also includes `runContext` metadata with repo revision, host fingerprint, DSN source mode, and active `BENCH_*` override values.
- `run_alpha_postgres_comparison_suite_repeats.sh` wraps the same suite for repeated runs (`--runs <n>`), snapshots each run’s artifacts to run-scoped files under `results/summaries/alpha-postgres-comparison-suite-runs/`, and writes one aggregate run-manifest summary (`results/summaries/alpha-postgres-comparison-suite-repeats.json`).
  - repeated summary now includes aggregated per-impl/per-endpoint stats across runs for `requestsPerSec`, `p99Ms`, and `rssKb` under `baselineStats` and `dbHotStats`.
- `run_alpha_postgres_comparison_suite_local.sh` orchestrates repo-local Postgres infra (`infra/local-postgres`) around the same two-phase suite and auto-injects DSN via temporary file, so local runs do not require manually exporting DSN flags.
- `run_alpha_postgres_comparison_suite_local_repeats.sh` orchestrates repo-local Postgres infra for repeated-run suites and delegates into the repeated wrapper with the same DSN auto-injection flow.
- `render_alpha_postgres_comparison_suite_repeats_summary.sh` converts repeated-suite JSON summary to markdown table report (`results/alpha-postgres-comparison-suite-repeats.md` by default).
- `run_workbench_smoke_matrix.sh` executes `benchmark-suite/workbench/matrix.backends.json` implementations with status `implemented-alpha|implemented`, runs each service smoke script, and writes `results/summaries/workbench-smoke-matrix.json`.
- `run_workbench_benchmark_matrix.sh` executes the same workbench matrix with load profiles (`wb-tasks-post`, `wb-tasks-with-comment`, `wb-task-comment-post`, `wb-task-get`, `wb-tasks-list`), writes per-impl summary/report artifacts, then emits compare/analysis/markdown report artifacts under `results/`.
  - also emits compact visual report: `results/workbench-benchmark-report.html`
  - `sec4-lasm` DB mode is configurable via:
    - `--lasm-db-adapter sqlite|postgres`
    - `--lasm-db-base <path>` (sqlite)
    - `--lasm-postgres-dsn-file <path>` (postgres)
- `run_workbench_benchmark_matrix_local.sh` orchestrates repo-local Postgres infra around workbench matrix fixed-target benchmarking and auto-injects `--lasm-db-adapter postgres` + temporary DSN file.
- `run_workbench_step_profile.sh` executes step-load rates for one workbench endpoint and writes:
  - per-rate snapshots: `results/summaries/<impl>-<endpoint>-r<rate>.json`
  - aggregate step summary: `results/summaries/<impl>-<endpoint>-step.json`
- `run_workbench_step_matrix.sh` executes workbench step-load profiles across matrix implementations/endpoints, runs step analysis per lane, and emits:
  - `results/summaries/workbench-step-runs.json`
  - `results/summaries/workbench-step-matrix.json`
- `run_workbench_step_matrix_local.sh` orchestrates repo-local Postgres infra around workbench step-load matrix runs and auto-injects `--lasm-db-adapter postgres` + temporary DSN file.
- `run_workbench_full_benchmark_suite.sh` runs workbench fixed-target matrix + workbench step matrix and republishes one combined markdown report with step-load signals:
  - `results/summaries/workbench-full-runs.json`
  - `results/workbench-full-benchmark-report.md`
  - `results/workbench-full-benchmark-report.html`
- `run_workbench_full_benchmark_suite_repeats.sh` wraps the full workbench suite repeatedly with run-scoped outputs and aggregate run manifest:
  - `results/summaries/workbench-full-benchmark-repeats.json`
  - `results/summaries/workbench-full-benchmark-runs/`
  - includes aggregate `compareStats` and `stepStats` across runs.
- `run_workbench_full_benchmark_suite_local.sh` orchestrates repo-local Postgres infra (`infra/local-postgres`) around the workbench full-suite runner and auto-injects `--lasm-db-adapter postgres` + temporary DSN file.
- `run_workbench_full_benchmark_suite_local_repeats.sh` orchestrates repo-local Postgres infra around repeated workbench full-suite runs and auto-injects `--lasm-db-adapter postgres` + temporary DSN file.
- `run_workbench_full_benchmark_suite_local_bundle.sh` runs local repeated-suite wrapper and renders markdown summary in one command (`workbench-full-benchmark-repeats.json` + `workbench-full-benchmark-repeats.md`).
- `render_workbench_full_benchmark_suite_repeats_summary.sh` converts repeated-run summary JSON to markdown:
  - `results/workbench-full-benchmark-repeats.md`
- `run_workbench_profile.sh` runs one workbench endpoint load profile and supports endpoint-specific targets:
  - `BENCH_TARGET_WB_TASKS_POST`
  - `BENCH_TARGET_WB_TASKS_WITH_COMMENT`
  - `BENCH_TARGET_WB_TASK_COMMENT_POST`
  - `BENCH_TARGET_WB_TASK_GET`
  - `BENCH_TARGET_WB_TASKS_LIST`
- Workbench mutation load profiles preserve the same alpha array payload contract across lanes (`params`, `task_params`, `comment_params`).
- Workbench endpoint profiles use deterministic auth + seeded task IDs; matrix runner seeds state via `/wb/setup` and one seed task/comment before load.
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
