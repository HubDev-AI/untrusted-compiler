# M39: LASM Postgres One-Shot Tx Fast Path And `O(1)` Record Compaction

## What it is

This slice closes the main remaining hot path on the canonical LASM Postgres workbench workload:

1. a one-shot Postgres `db.execTx` fast path for non-retained transactional routes
2. narrower shared-pool wakeups
3. `O(1)` cleanup for dropped DB record signatures in dynamic state

Files:

- `compiler/sec4-cli/src/lasm_db_runtime_postgres.rs`
- `compiler/sec4-cli/src/lasm_db_client/operations.rs`
- `compiler/sec4-cli/src/lasm_dynamic_state.rs`

## Why it exists

The canonical workbench app had already proven that:

- the benchmark harness was no longer the main blocker
- the ordinary one-statement route was healthy
- the explicit tx route was still the highest-value remaining runtime hotspot

At that point, more orchestration tweaks were lower leverage than reducing per-request transaction overhead and global-lock hold time on the real Postgres path.

## How it works internally

### 1. One-shot `db.execTx` fast path

For non-retained transactional execution, LASM now avoids the heavier savepoint-style retained-tx lifecycle.

The runtime now:

1. checks out a detached tx client
2. validates the query before `BEGIN`
3. runs `BEGIN`
4. executes the statement
5. commits with `COMMIT`
6. returns or discards the client based on cleanup safety

Retained multi-step tx handles still use the existing savepoint path. The fast path is only for the single-request case where the tx handle does not need to survive across operations.

### 2. Shared-pool wakeup narrowing

Shared Postgres pool release/connect-failure paths now use `notify_one()` instead of `notify_all()`.

That reduces unnecessary wakeups when multiple workers are waiting on the same pool key.

### 3. `O(1)` record-signature compaction cleanup

`append_lasm_dynamic_db_record(...)` used to reverse-scan the full record history when the latest record for a signature was dropped during compaction.

Now, when the signature reference count reaches zero, runtime state removes the latest-signature index entry directly instead of scanning history.

That shortens mutex hold time on the write-heavy path.

## Inputs, outputs, and constraints

Inputs:

- canonical LASM workbench Postgres routes
- non-retained `db.execTx` runtime execution
- dynamic DB record compaction under load

Outputs:

- same public HTTP contract
- same Postgres SQL contract
- lower tx-route overhead and less global-lock churn

Constraints:

- retained tx-handle semantics must not change
- diagnostics and envelopes must stay deterministic
- the canonical workbench smoke and benchmark contract must remain unchanged

## Failure modes and diagnostics

- If query validation fails, runtime still returns the same deterministic DB validation error path.
- If `BEGIN`, execution, or `COMMIT` fails, runtime still reports the same operation-class error with safe client discard when cleanup safety is uncertain.
- The one-shot fast path does not change public route behavior. It only changes how the runtime executes the transaction internally.

## Example usage

This fast path is exercised by the canonical explicit-tx route:

- `POST /wb/tasks/with-comment-tx`

Validation surface used in this slice:

```bash
cargo build -p sec4
benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh
BENCH_WORKBENCH_REQUIRE_WRK2=1 \
benchmark-suite/scripts/run_workbench_step_matrix.sh \
  --impls sec4-lasm \
  --endpoints wb-tasks-with-comment-tx \
  --lasm-db-adapter postgres \
  --lasm-mode auto \
  --port 18251
```

## Measured result

Focused explicit-tx step rerun on the canonical Postgres workload:

- before: about `443.68 req/s`, `p99 3.76s` at the `500` target lane
- after: about `489.63 req/s`, `p99 154.75ms`

Current tuned fixed compare on Postgres:

- `wb-tasks-with-comment`: about `198.35 req/s`, `p99 36.54ms`
- `wb-tasks-with-comment-tx`: about `199.08 req/s`, `p99 41.22ms`

## Tradeoffs and next steps

This was the right optimization because it removes real tx-path cost without changing public semantics.

Tradeoffs:

- runtime code now carries two Postgres tx execution shapes:
  - retained multi-step path
  - one-shot non-retained path
- that split is justified because the workloads are materially different and the canonical app exposed the cost clearly

Next step:

- stop iterating on this exact tx hotspot unless new evidence shows regression
- use the tuned workbench baseline as the regression target while extracting remaining LASM DB adapter layers into packages/modules without semantic changes
