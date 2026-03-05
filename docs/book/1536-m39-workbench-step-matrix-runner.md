# 1536 M39 Slice: Workbench Step Matrix Runner

This slice adds step-load orchestration for workbench benchmark lanes so knee signals can be compared across implementations under the same feature-app contract.

## What changed

1. Added staged-rate workbench step-profile runner:
   - `benchmark-suite/scripts/run_workbench_step_profile.sh`
   - runs one implementation/endpoint over configured step rates and emits:
     - per-rate snapshots: `results/summaries/<impl>-<endpoint>-r<rate>.json`
     - aggregate step summary: `results/summaries/<impl>-<endpoint>-step.json`
2. Added full workbench step-matrix orchestrator:
   - `benchmark-suite/scripts/run_workbench_step_matrix.sh`
   - behavior:
     - selects implemented workbench lanes from `benchmark-suite/workbench/matrix.backends.json`,
     - starts each lane, waits for readiness, runs deterministic setup/seed,
     - executes `run_workbench_step_profile.sh` per endpoint,
     - runs `analyze_step_profile.sh` for each lane/endpoint,
     - emits cross-implementation step matrix with `compare_step_matrix.sh`.
3. Added make targets:
   - `make -C benchmark-suite workbench-step-bench-dry`
   - `make -C benchmark-suite workbench-step-bench`
4. Added workbench step-runner support for LASM DB mode controls:
   - `WORKBENCH_LASM_DB_ADAPTER=sqlite|postgres`
   - `WORKBENCH_LASM_DB_BASE=<path>` (sqlite mode)
   - `WORKBENCH_LASM_DB_POSTGRES_DSN_FILE=<path>` (postgres mode)
5. Updated benchmark docs:
   - `benchmark-suite/README.md`
   - `benchmark-suite/workbench/README.md`
   - roadmap and workbench plan snapshots.

## Why

Workbench fixed-target matrix runs already existed, but there was no dedicated way to compare step-load knee behavior for the same feature-app endpoints. This slice closes that gap and keeps the same DB-mode controls so LASM sqlite/postgres modes can be evaluated under both fixed-target and staged-rate runs.

## Validation

1. `bash -n benchmark-suite/scripts/run_workbench_step_profile.sh`
2. `bash -n benchmark-suite/scripts/run_workbench_step_matrix.sh`
3. `make -C benchmark-suite workbench-step-bench-dry`
4. `BENCH_STEP_DURATION=2s BENCH_STEP_RATES=80,120 benchmark-suite/scripts/run_workbench_step_matrix.sh --impls sec4 --endpoints wb-task-get --port 18096`
