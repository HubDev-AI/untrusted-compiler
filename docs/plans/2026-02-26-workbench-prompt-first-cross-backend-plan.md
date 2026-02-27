# Workbench Plan: Prompt-First Cross-Backend Benchmark

Date: 2026-02-26  
Owner: sec4 core/runtime lane

## Goal

Build one larger, realistic backend app contract with real DB behavior, generate equivalent implementations across multiple backends from a single prompt contract, and benchmark all implementations (including sec4/sec4-lasm) under the same conditions.

## Scope

1. One canonical "feature-rich API" contract:
   - auth-required endpoints
   - read/write DB paths
   - transaction path
   - filtered list/search path
   - deterministic error envelopes
2. Prompt-first generation workflow:
   - one canonical generation prompt
   - backend-specific adaptation prompt
   - deterministic acceptance checklist
3. Benchmark execution through existing `benchmark-suite` runners and artifact contracts.

## Out of Scope (for this slice)

1. Full automation of AI generation loop in CI.
2. Performance tuning campaign.
3. New language semantics.

## Backend Matrix (initial)

1. `sec4` (C runtime path)
2. `sec4-lasm`
3. `node`
4. `go`
5. `rust`

## Workbench App Contract (v1)

The app is a "task board + comments + labels" API with real DB usage.

Required characteristics:

1. Auth gate:
   - `Authorization: Bearer <token>` required for all mutating endpoints.
2. DB write path:
   - create task
   - create comment
3. DB read path:
   - get task by id
   - list tasks with pagination/filter
4. Transaction path:
   - create task + initial comment atomically.
5. Deterministic response envelope:
   - success: `{ ok, status, traceId, timeMs, data }`
   - error: `{ ok:false, status, traceId, timeMs, error:{ code, kind, message } }`

## Prompt-First Generation Flow

### Phase A: Contract freeze

1. Write canonical API + DB schema contract (`benchmark-suite/workbench/spec/feature-app-v1.md`).
2. Freeze deterministic envelope + validation/error codes.
3. Freeze benchmark request mix and payload set.

### Phase B: Generate implementations

1. Use canonical prompt (`benchmark-suite/workbench/prompts/generate-feature-app-v1.md`).
2. Generate one implementation per backend into:
   - `benchmark-suite/services/<impl>-workbench/`
3. Run backend-specific smoke checks.
4. Reject generated code that violates contract or introduces placeholder behavior.

### Phase C: Benchmark and compare

1. Add workbench endpoint profiles/load scripts.
2. Run fixed-target and step profiles across all implementations.
3. Build compare matrix and report using current artifact flow.
4. Publish summary with throughput, p95/p99, error-rate, and RSS.

## Execution Order

1. Define/freeze contract and generation prompt.
2. Implement sec4/sec4-lasm reference first.
3. Generate node/go/rust against frozen prompt and contract.
4. Run parity checks.
5. Run benchmark matrix.

## Guardrails

1. No per-backend feature drift; contract parity is mandatory.
2. No synthetic in-memory fake DB for DB endpoints; real Postgres path required for matrix runs.
3. Deterministic error envelopes across all backends.
4. Same load profile parameters and host constraints across implementations.

## Deliverables

1. `benchmark-suite/workbench/spec/feature-app-v1.md`
2. `benchmark-suite/workbench/prompts/generate-feature-app-v1.md`
3. `benchmark-suite/workbench/matrix.backends.json`
4. Benchmark report section: "Workbench Feature App Matrix"

## Immediate Next Steps

1. Land workbench scaffold files (contract + prompts + matrix config).
2. Add sec4/sec4-lasm workbench service skeletons.
3. Add one dry-run generation pass for node/go/rust.

## Progress Snapshot (2026-02-26)

1. Completed:
   - workbench scaffold files landed.
   - `sec4-lasm` workbench service implemented:
     - `benchmark-suite/services/sec4-lasm-workbench`
     - real auth-gated DB routes (`db.exec`, `db.execTx`, `db.queryOne`).
   - `node` workbench service implemented:
     - `benchmark-suite/services/node-workbench`
     - real Postgres-backed task/comment routes with deterministic envelope contract.
   - `go` workbench service implemented:
     - `benchmark-suite/services/go-workbench`
     - real Postgres-backed task/comment routes with deterministic envelope contract.
   - `rust` workbench service implemented:
     - `benchmark-suite/services/rust-workbench`
     - real Postgres-backed task/comment routes with deterministic envelope contract.
   - `sec4` workbench service implemented:
     - `benchmark-suite/services/sec4-workbench`
     - c-backend execution of the same route contract with deterministic runtime envelope checks.
   - c-backend emit unblock for status-form JSON responses:
     - `res.json(status, schema, value)` now lowers to `sec4_rt_res_ok(...)` on C emit paths.
   - wire-format parity across all implementations:
     - `node`, `go`, and `rust` lanes now accept the same alpha benchmark query payload keys used by sec4 lanes (`params`, `task_params`, `comment_params`).
   - workbench smoke orchestrator implemented:
      - `benchmark-suite/scripts/run_workbench_smoke_matrix.sh`
      - `make -C benchmark-suite workbench-smoke`
   - workbench benchmark matrix runner implemented:
      - endpoint load profiles added (`wb-tasks-post`, `wb-tasks-with-comment`, `wb-task-comment-post`, `wb-task-get`, `wb-tasks-list`)
      - `benchmark-suite/scripts/run_workbench_profile.sh`
      - `benchmark-suite/scripts/run_workbench_benchmark_matrix.sh`
      - make targets:
        - `make -C benchmark-suite workbench-bench-dry`
        - `make -C benchmark-suite workbench-bench`
      - output artifacts:
        - `benchmark-suite/results/summaries/workbench-benchmark-runs.json`
        - `benchmark-suite/results/summaries/workbench-benchmark-compare-matrix.json`
        - `benchmark-suite/results/summaries/workbench-benchmark-analysis.json`
        - `benchmark-suite/results/workbench-benchmark-report.md`
   - workbench LASM DB mode controls and Postgres compatibility hardening:
      - `run_workbench_benchmark_matrix.sh` now supports `--lasm-db-adapter sqlite|postgres`, `--lasm-db-base`, and `--lasm-postgres-dsn-file`,
      - make-level passthrough knobs added (`WORKBENCH_LASM_DB_ADAPTER`, `WORKBENCH_LASM_DB_BASE`, `WORKBENCH_LASM_DB_POSTGRES_DSN_FILE`),
      - workbench task insert templates now cast priority placeholder (`cast($5 as bigint)`) to remove LASM Postgres write-path serialization failure.
   - workbench step-load matrix runner implemented:
      - `benchmark-suite/scripts/run_workbench_step_profile.sh`
      - `benchmark-suite/scripts/run_workbench_step_matrix.sh`
      - make targets:
        - `make -C benchmark-suite workbench-step-bench-dry`
        - `make -C benchmark-suite workbench-step-bench`
      - output artifacts:
        - `benchmark-suite/results/summaries/workbench-step-runs.json`
        - `benchmark-suite/results/summaries/workbench-step-matrix.json`
        - per-lane step summaries + analyses (`<impl>-<endpoint>-step.json`, `<impl>-<endpoint>-step-analysis.json`).
   - workbench full benchmark suite runner implemented:
      - `benchmark-suite/scripts/run_workbench_full_benchmark_suite.sh`
      - make targets:
        - `make -C benchmark-suite workbench-full-bench-dry`
        - `make -C benchmark-suite workbench-full-bench`
      - output artifacts:
        - `benchmark-suite/results/summaries/workbench-full-runs.json`
        - `benchmark-suite/results/workbench-full-benchmark-report.md`
      - runs fixed-target matrix + step matrix and republishes one combined report with step-load signals.
   - repeated full-suite wrapper implemented:
      - `benchmark-suite/scripts/run_workbench_full_benchmark_suite_repeats.sh`
      - make targets:
        - `make -C benchmark-suite workbench-full-bench-repeats-dry`
        - `make -C benchmark-suite workbench-full-bench-repeats`
      - output artifacts:
        - `benchmark-suite/results/summaries/workbench-full-benchmark-repeats.json`
        - `benchmark-suite/results/summaries/workbench-full-benchmark-runs/`
   - repeated-run aggregate stats + markdown summary implemented:
      - repeated JSON summary now includes aggregate `compareStats` and `stepStats` fields across runs,
      - renderer script:
        - `benchmark-suite/scripts/render_workbench_full_benchmark_suite_repeats_summary.sh`
      - make target:
        - `make -C benchmark-suite workbench-full-bench-repeats-report`
      - markdown output:
        - `benchmark-suite/results/workbench-full-benchmark-repeats.md`
   - benchmark-smoke CI guard coverage expanded for workbench full-suite lanes:
      - added script tests:
        - `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite.sh`
        - `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite_repeats.sh`
        - `benchmark-suite/scripts/test_render_workbench_full_benchmark_suite_repeats_summary.sh`
      - added to benchmark smoke workflow and `make -C benchmark-suite test-scripts` run list.
   - local Postgres infra wrappers added for workbench full-suite lanes:
      - `benchmark-suite/scripts/run_workbench_full_benchmark_suite_local.sh`
      - `benchmark-suite/scripts/run_workbench_full_benchmark_suite_local_repeats.sh`
      - wrappers auto-wire `--lasm-db-adapter postgres` and temp DSN file resolution from `infra/local-postgres`,
      - make targets:
        - `make -C benchmark-suite workbench-full-bench-local[-dry]`
        - `make -C benchmark-suite workbench-full-bench-local-repeats[-dry]`
      - benchmark-smoke CI now runs:
        - `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite_local.sh`
        - `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite_local_repeats.sh`
   - local-infra full-suite bundle wrapper added (repeats + markdown render in one command):
      - `benchmark-suite/scripts/run_workbench_full_benchmark_suite_local_bundle.sh`
      - make targets:
        - `make -C benchmark-suite workbench-full-bench-local-bundle[-dry]`
      - benchmark-smoke CI now runs:
        - `benchmark-suite/scripts/test_run_workbench_full_benchmark_suite_local_bundle.sh`
2. Current matrix state:
   - `sec4-lasm`: `implemented-alpha` and passing smoke.
   - `node`: `implemented-alpha` and passing smoke.
   - `go`: `implemented-alpha` and passing smoke.
   - `rust`: `implemented-alpha` and passing smoke.
   - `sec4`: `implemented-alpha` and passing smoke.
