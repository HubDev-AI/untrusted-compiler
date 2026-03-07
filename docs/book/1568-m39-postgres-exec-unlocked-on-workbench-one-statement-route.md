# M39: Postgres `db.exec` Unlocked On Workbench One-Statement Route

## What changed

The Postgres `db.exec` dispatch path no longer runs the thread-local execution call while holding the global LASM dynamic-state mutex.

Files:

- `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`

## Why this mattered

The canonical workbench app exposed a clear split:

- explicit tx route `POST /wb/tasks/with-comment-tx` was healthy
- one-statement route `POST /wb/tasks/with-comment` was much slower

That ruled out the benchmark harness and pointed at the ordinary Postgres `db.exec` path.

The old dispatch shape still did this under the global mutex:

1. validate adapter state
2. build Postgres config
3. execute the thread-local Postgres operation
4. append runtime record

Step 3 was the problem. `run_lasm_postgres_exec_thread_local(...)` does not need the global dynamic state, but dispatch kept the mutex held across that call.

## What changed in dispatch

For Postgres `db.exec`, dispatch now:

1. locks only to validate adapter state and build `LasmPostgresThreadLocalConfig`
2. unlocks before `run_lasm_postgres_exec_thread_local(...)`
3. re-locks only to append the runtime record and schedule persistence

Other adapters keep their existing path.

## Validation

Validated with:

```bash
cargo build -p sec4
benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh
BENCH_DURATION=1s BENCH_WORKBENCH_REQUIRE_WRK2=1 \
  benchmark-suite/scripts/run_workbench_benchmark_matrix.sh \
  --impls sec4-lasm \
  --endpoints wb-tasks-with-comment \
  --lasm-db-adapter postgres \
  --port 18124
```

## Result

Focused Postgres rerun on `wb-tasks-with-comment`:

- before: about `160.36 req/s`, `p99 586.24ms`
- after: about `347.26 req/s`, `p99 109.82ms`

Correctness stayed green:

- `smoke-public.sh` passed
- benchmark run passed with no socket-error failure path

## Why this is the right next step

This is the same pattern that already paid off on `execTx`:

- keep the global dynamic-state lock for bookkeeping only
- do the real Postgres network/statement work outside that lock

It is a smaller and safer move than the larger `execTx` dispatch collapse that still remains as the next structural cleanup item.
