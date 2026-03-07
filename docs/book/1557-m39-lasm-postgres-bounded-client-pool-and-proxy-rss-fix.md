# M39: LASM Postgres Bounded Client Pool and Proxy RSS Fix

## What changed

Two real problems surfaced under the canonical Postgres-backed workbench benchmark:

1. LASM Postgres runtime paths could open too many concurrent clients under cluster load and hit `sqlstate=53300` (`too many clients already`).
2. Proxy-cluster benchmark RSS sampling could report `null` because the harness trusted a stale launcher PID instead of the live listener/process tree.

This slice fixed both.

## Runtime fix

Files:
- `compiler/sec4-cli/src/lasm_db_runtime_postgres.rs`
- `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`

Changes:
- Replaced idle-only reuse with a bounded shared Postgres client pool that tracks active checkouts as well as idle clients.
- Added wait/reuse behavior so concurrent work blocks instead of stampeding Postgres with unbounded client creation.
- Routed transaction-client acquisition through the same bounded pool.
- Added explicit discard paths so failed tx clients release active-pool slots instead of leaking them.

Result:
- `wb-tasks-with-comment` no longer degrades into `too many clients already` failures under the short cluster benchmark path.

## Benchmark harness fix

Files:
- `benchmark-suite/scripts/run_workbench_profile.sh`
- `benchmark-suite/scripts/run_profile.sh`
- `benchmark-suite/scripts/run_workbench_benchmark_matrix.sh`
- `benchmark-suite/scripts/run_workbench_step_matrix.sh`

Changes:
- RSS sampling now sums the full process tree per PID instead of relying on one batched `ps -p` call.
- Benchmark profile scripts can fall back to resolving the live listener PID from `BENCH_SERVER_PORT` when the tracked launcher PID is stale.
- Workbench matrix runners now pass `BENCH_SERVER_PORT` and resolve the real listener PID for LASM proxy-cluster mode before profiling.

Result:
- proxy-cluster workbench runs now emit real `rssKb` values instead of `null`.

## Validation

Ran:

```bash
cargo check -p sec4
```

And re-ran the real short canonical comparison:

```bash
BENCH_DURATION=1s BENCH_WORKBENCH_REQUIRE_WRK2=0 \
benchmark-suite/scripts/run_workbench_lasm_mode_compare.sh \
  --endpoints wb-tasks-with-comment \
  --lasm-db-adapter postgres \
  --port 18454 \
  --instances 2 \
  --autoscale-max-instances 4 \
  --cluster-accept-workers 2 \
  --cluster-relay-pump-batch-max 128
```

Observed artifact:
- `benchmark-suite/results/summaries/workbench-lasm-mode-compare.json`

Latest result snapshot:
- `single`: ~`349.56 rps`, `p99 133.50ms`, `rss 44240 KB`
- `fixed`: ~`350.26 rps`, `p99 140.54ms`, `rss 58400 KB`
- `proxy`: ~`346.23 rps`, `p99 10.37ms`, `rss 49152 KB`

## Why this matters

This closes a real alpha-quality gap:
- the runtime now behaves predictably against a real Postgres ceiling,
- and the scaling benchmark now measures proxy memory honestly instead of silently dropping it.

That keeps the next scaling/runtime tuning pass grounded in real numbers from the canonical app.
