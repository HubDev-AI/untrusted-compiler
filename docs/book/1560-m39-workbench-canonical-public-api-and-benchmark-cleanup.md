# M39: Workbench canonical public API and benchmark cleanup

## What changed

1. `benchmark-suite/services/sec4-lasm-workbench/src/workbench/tasks.ut`
   - `POST /wb/tasks` now runs a second DB write to persist labels in `wb_labels`.
   - `GET /wb/tasks/:id` now returns `labels` alongside the existing task payload.
   - `GET /wb/tasks` now supports the public filter shape through one expanded params contract:
     - `status`
     - `priorityMin`
     - `priorityMax`
     - `label`
     - `limit`
     - `offset`

2. `compiler/sec4-cli/src/lasm_request_template.rs`
   - LASM request ingestion now synthesizes benchmark-internal query params from the public route shape:
     - JSON body -> `params`
     - JSON body -> `task_params`
     - JSON body -> `comment_params`
     - JSON body/query -> `label_params`
     - public list filters -> expanded list `params`
     - missing public `row_schema` for workbench GET routes -> default `1`
   - The old benchmark wire format remains accepted, including automatic expansion of the older 3-element list array into the new 6-element form.

3. `compiler/sec4-cli/src/main.rs`
   - workbench error normalization now maps comment writes against a missing task to:
     - HTTP `404`
     - `TASK.NOT_FOUND`
     - `missing_dependency`

4. `benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh`
   - Added a real public smoke flow for the canonical JSON API on Postgres/LASM:
     - setup
     - create task with labels
     - transactional create with initial comment
     - add comment
     - get task without `row_schema`
     - filtered list without legacy `params`
     - deterministic missing-task comment failure

5. `benchmark-suite/services/sec4-lasm-workbench/README.md`
   - Updated to describe the public route contract and both smoke entrypoints.

6. `benchmark-suite/scripts/run_workbench_benchmark_matrix.sh`
   - benchmark cleanup now kills service process trees and waits for the listener port to drain before the next impl starts, removing the false Rust `AddrInUse` failure mode.

## Why

The workbench service had become benchmark-capable but still required internal alpha knobs (`params`, `task_params`, `row_schema`) for normal use. That was not a credible canonical app.

This slice keeps benchmark parity while making the LASM workbench usable as an operator-facing app:

1. Public JSON bodies and public list filters work directly.
2. The old cross-backend benchmark contract still works unchanged.
3. Missing-task comment writes now surface as domain errors instead of raw DB failures.

## Validation

1. `cargo build -p sec4`
2. `target/debug/sec4 check --path benchmark-suite/services/sec4-lasm-workbench`
3. `benchmark-suite/services/sec4-lasm-workbench/smoke.sh`
4. `benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh`
5. Real Postgres public flow:
   - `POST /wb/tasks`
   - `POST /wb/tasks/with-comment`
   - `POST /wb/tasks/:id/comments`
   - `GET /wb/tasks/:id`
   - `GET /wb/tasks?status=open&label=api&limit=10&offset=0`
6. Same-workload benchmark rerun:
   - `sec4-lasm` passed all endpoints
   - `node` passed all endpoints in the comparison run
   - Rust false `AddrInUse` failure path was removed by process-tree/port-drain cleanup
   - Go still emits non-2xx responses on `wb-tasks-post` under the current workload, which now appears to be a real baseline-service limitation rather than matrix contamination

## Tradeoff

The workbench public API is still implemented by synthesizing the alpha benchmark wire contract behind the scenes. That is the correct short-term move because it keeps the benchmark suite stable while making the LASM app usable now. A future cleanup pass can remove that compatibility layer once the shared benchmark request generator speaks the public contract directly.
