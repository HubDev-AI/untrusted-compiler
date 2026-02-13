# 224 M10 Slice: Cross-Implementation Benchmark Orchestrator

This chapter documents M10 orchestration for running the benchmark matrix across implementations with one command.

## What it is

Added:
- `benchmark-suite/scripts/run_comparison_matrix.sh`
- `benchmark-suite/scripts/test_run_comparison_matrix.sh`
- Make targets:
  - `make -C benchmark-suite bench-matrix`
  - `make -C benchmark-suite bench-matrix-dry`

The orchestrator runs per-impl benchmark profiles, builds implementation reports, and emits matrix + markdown report outputs.

## Why it exists

M10 requires reproducible cross-language benchmarking, not manual ad hoc command chains. This slice standardizes the full comparison flow and reduces operator error.

## How it works internally

1. Select implementations (default: `node,go,rust`).
2. For each implementation:
   - starts the service on `:8080`,
   - waits for `/ping` readiness,
   - runs `ping`, `decode`, `users-post` profiles,
   - builds `<impl>-report.json`,
   - stops service process.
3. After all implementations finish:
   - builds `compare-matrix.json`,
   - publishes markdown benchmark report.

`--dry-run` prints the full command plan without running load tests.

## Inputs, outputs, and constraints

- Inputs:
  - service runners for selected implementations,
  - benchmark scripts (`run_profile`, `build_report`, `compare_matrix`, `publish_report`).
- Outputs:
  - `results/summaries/<impl>-report.json`
  - `results/summaries/compare-matrix.json`
  - `results/benchmark-report.md`
- Constraints:
  - currently supports `node`, `go`, and `rust` only,
  - assumes services can run on local port `8080` sequentially.

## Failure modes and diagnostics

- unsupported implementation -> explicit error.
- missing service directory -> explicit error.
- readiness timeout (`/ping`) -> explicit error.
- downstream script failures propagate and stop orchestration.

## Example usage

Dry run:

```bash
make -C benchmark-suite bench-matrix-dry
```

Full run:

```bash
make -C benchmark-suite bench-matrix
```

Custom implementations:

```bash
benchmark-suite/scripts/run_comparison_matrix.sh --impls node,go
```

## Tradeoffs and next steps

- Tradeoff:
  - orchestrator currently runs services sequentially on one port for deterministic local execution, not parallelized multi-host execution.
- Next:
  - add AILang service integration once runtime HTTP path reaches benchmark-ready behavior,
  - add optional C floor comparator integration,
  - support configurable endpoint sets and alternative load profiles.
