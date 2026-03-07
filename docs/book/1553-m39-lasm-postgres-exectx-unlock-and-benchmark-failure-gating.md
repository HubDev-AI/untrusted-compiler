# M39: LASM Postgres `db.execTx` Unlock And Benchmark Failure Gating

## 1. What It Is

This slice changes the LASM Postgres `db.execTx` runtime so the expensive transactional work no longer runs while holding the global dynamic response-state mutex.

It also hardens the workbench benchmark harness so a run fails when:

- zero requests complete, or
- `wrk2` reports socket errors (`connect`, `read`, `write`, or `timeout`).

## 2. Why It Exists

The canonical workbench route `POST /wb/tasks/with-comment` is the real transactional write path for alpha.

Under sustained `wrk2` load, the route previously degraded into timeout-only runs because:

1. tx-handle bookkeeping and Postgres client I/O were serialized under the same mutex, and
2. the benchmark harness only failed on non-2xx/3xx counts, so timeout-only runs could look green.

That meant the app looked healthier than it was and the runtime could not be trusted on the benchmark workload that now defines the alpha proof path.

## 3. How It Works Internally

### Runtime path

`compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs` now special-cases the Postgres `execTx` path:

1. lock global state,
2. validate tx/db bindings,
3. mark the tx handle as `in_use`,
4. collect config and detach any per-tx Postgres client,
5. unlock global state,
6. run connect/exec/commit/rollback on the detached client,
7. re-lock global state only for tx-state updates and record append/persistence scheduling.

`compiler/sec4-cli/src/lasm_dynamic_state.rs` adds `in_use` to `LasmDbTxState` so an existing tx handle cannot be borrowed concurrently by another request while its client is detached.

`compiler/sec4-cli/src/lasm_db_runtime_postgres.rs` now exposes explicit helpers for:

- detached tx-client connect,
- detached tx-client take/put,
- detached `execTx`,
- detached commit/rollback.

### Benchmark gating

`benchmark-suite/scripts/wrk2_summary.sh` now parses:

- `completedRequests`,
- socket error counts,
- existing response and latency fields.

`benchmark-suite/scripts/run_workbench_profile.sh` now fails the run when:

- `completedRequests == 0`, or
- total socket errors is non-zero, or
- non-2xx/3xx responses are non-zero.

## 4. Inputs, Outputs, And Constraints

### Inputs

- canonical workbench route `POST /wb/tasks/with-comment`,
- LASM Postgres runtime config,
- benchmark raw output from `wrk2`.

### Outputs

- successful Postgres transactional execution without timeout-only collapse,
- deterministic tx-handle ownership while the client is detached,
- benchmark summaries that distinguish real success from transport failure.

### Constraints

- tx-handle semantics remain runtime-ephemeral,
- record append/persistence behavior stays deterministic,
- benchmark contract remains shared across `sec4-lasm`, `node`, `go`, and `rust`.

## 5. Failure Modes And Diagnostics

Runtime still returns normal DB envelopes on real SQL/runtime failures.

New deterministic runtime guard:

- reusing an already borrowed tx handle now returns `409 DB.EXEC_TX_CONFLICT`.

Benchmark failures now surface directly instead of hiding inside a “passed” summary:

- `completedRequests=0`,
- `socketErrors>0`,
- existing non-2xx/3xx checks.

## 6. Example Usage

Validate the canonical transactional route on Postgres:

```bash
set -a && source infra/local-postgres/.runtime.env && set +a
target/debug/sec4 run \
  --path benchmark-suite/services/sec4-lasm-workbench \
  --backend lasm \
  --db-adapter postgres \
  --port 18410
```

Then profile the failing lane directly:

```bash
BENCH_DURATION=1s BENCH_REQUIRE_WRK2=0 \
benchmark-suite/scripts/run_workbench_profile.sh \
  sec4-lasm wb-tasks-with-comment http://127.0.0.1:18410
```

Run the canonical smoke with the built binary:

```bash
benchmark-suite/services/sec4-lasm-workbench/smoke.sh
```

## 7. Tradeoffs And Next Steps

Tradeoffs:

- the global mutex is still used for authoritative runtime bookkeeping,
- throughput is improved materially, but this is not the end-state scaling pass,
- tx-handle `in_use` tracking adds one more explicit state bit in exchange for correctness.

Next steps:

1. rerun the full DB-backed matrix against `node`, `go`, and `rust`,
2. close the next DB/runtime cleanup surfaced by the canonical workbench app,
3. then start the planned scaling/runtime tuning pass on the same workload.
