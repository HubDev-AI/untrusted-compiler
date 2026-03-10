# 05 - Benchmarks and Capacity

This page covers reproducible benchmark workflows across:

- `sec4`
- `sec4-lasm`
- `node`
- `go`
- `rust`
- optional `c` (non-primary lane)

Primary reference:

- `benchmark-suite/README.md`
- canonical workload app: `benchmark-suite/services/sec4-lasm-workbench`

## 1) Benchmark prerequisites

```bash
make -C benchmark-suite db-up
make -C benchmark-suite db-schema
make -C benchmark-suite preflight
```

If `wrk2` is missing:

```bash
make -C benchmark-suite wrk2-install
export BENCH_WRK2_BIN="$PWD/benchmark-suite/bin/wrk2"
```

## 2) Run matrix comparison (legacy generic lane)

Dry run:

```bash
make -C benchmark-suite bench-matrix-dry
```

Real run:

```bash
make -C benchmark-suite bench-matrix
```

Generate report:

```bash
make -C benchmark-suite publish-report
```

## 3) Canonical cross-backend matrix (post-alpha)

This is the primary comparison command for the canonical workbench contract.
It runs the same DB-backed endpoint set across `sec4-lasm,node,go,rust` and publishes the report bundle.

Dry run:

```bash
benchmark-suite/scripts/run_workbench_benchmark_matrix_local.sh \
  --dry-run \
  --impls sec4-lasm,node,go,rust \
  --lasm-mode single \
  --fail-on-impl-failure 0 \
  --profile-retry-on-failure 0 \
  --lasm-max-keep-alive-requests 4096
```

Real run:

```bash
BENCH_DURATION=20s \
benchmark-suite/scripts/run_workbench_benchmark_matrix_local.sh \
  --impls sec4-lasm,node,go,rust \
  --lasm-mode single \
  --fail-on-impl-failure 0 \
  --profile-retry-on-failure 0 \
  --lasm-max-keep-alive-requests 4096
```

Outputs:

- `benchmark-suite/results/workbench-benchmark-report.md`
- `benchmark-suite/results/workbench-benchmark-report.html`
- `benchmark-suite/results/summaries/workbench-benchmark-runs.json`
- `benchmark-suite/results/summaries/workbench-benchmark-compare-matrix.json`
- `benchmark-suite/results/summaries/workbench-benchmark-analysis.json`

Strict gate mode (fail immediately when any impl lane fails):

```bash
benchmark-suite/scripts/run_workbench_benchmark_matrix_local.sh \
  --impls sec4-lasm,node,go,rust \
  --lasm-mode single \
  --fail-on-impl-failure 1
```

## 4) Capacity probe (1M requests target)

sec4:

```bash
make -C benchmark-suite sec4-capacity-probe CAPACITY_TARGET_REQUESTS=1000000
```

LASM cluster:

```bash
make -C benchmark-suite lasm-cluster-capacity-probe LASM_CAPACITY_TARGET_REQUESTS=1000000
```

## 5) Full-suite/repeat wrappers (optional)

```bash
make -C benchmark-suite bench-alpha-postgres-suite-local
```

Repeated runs:

```bash
make -C benchmark-suite bench-alpha-postgres-suite-local-repeats BENCH_ALPHA_POSTGRES_REPEAT_RUNS=3
```

## 6) Stop infra

```bash
make -C benchmark-suite db-down
```

## Next step

- Continue to `06-troubleshooting.md`.
