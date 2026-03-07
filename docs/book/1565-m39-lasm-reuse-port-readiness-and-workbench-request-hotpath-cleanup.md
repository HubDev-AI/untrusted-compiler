# M39 Slice: LASM Reuse-Port Readiness And Workbench Request Hot-Path Cleanup

## What It Is

This slice closes two real issues that were still sitting on the canonical LASM/Postgres path:

1. fixed reuse-port cluster workers were treated as "ready" when they were only alive
2. workbench request augmentation was doing unnecessary work on requests that never needed workbench JSON synthesis

Both issues live below the language surface, on the runtime/operator path.

## Why It Exists

The fixed reuse-port cluster path is part of the LASM scaling model. If the parent process only waits for child liveness, short operator benchmarks and startup checks can observe a cluster before all workers are actually serving.

Separately, the request bridge for the canonical workbench app had grown extra per-request cost:

- it parsed JSON bodies before checking whether the route even needed workbench body augmentation
- it re-read `SEC4_RT_DEBUG_WORKBENCH_JSON` from the environment on every request/response pass

Those are not correctness bugs in the language, but they are real runtime costs and they distort scaling/runtime tuning.

## What Changed

### 1. Reuse-port worker readiness is now explicit

Files:

- `compiler/sec4-cli/src/lasm_cluster_lifecycle.rs`
- `compiler/sec4-cli/src/main.rs`

The parent now creates a per-worker readiness file path and passes it through:

- `SEC4_RT_LASM_READY_FILE`

The child LASM process writes that file only after:

1. listener bind succeeded
2. runtime state initialized
3. worker pool setup completed

The fixed reuse-port cluster parent now waits on that readiness signal instead of using the old "still alive after grace period" heuristic.

### 2. Workbench query augmentation now route-gates body parsing

File:

- `compiler/sec4-cli/src/lasm_request_template.rs`

Before this slice, `augment_lasm_workbench_query_params_from_body(...)` parsed the JSON body up front whenever the request had a JSON body, even if the route was unrelated.

Now it only parses the body inside the specific route branches that actually need synthesized params:

- `POST /wb/tasks`
- `POST /wb/tasks/with-comment`
- `POST /wb/tasks/with-comment-tx`
- `POST /wb/tasks/:id/comments`

### 3. Workbench debug env lookup is cached

File:

- `compiler/sec4-cli/src/main.rs`

The runtime now resolves `SEC4_RT_DEBUG_WORKBENCH_JSON` once through `OnceLock<bool>` and reuses that cached decision in both:

- request debug logging
- response debug logging

### 4. Benchmark script now preflights the wrk template renderer

File:

- `benchmark-suite/scripts/run_workbench_profile.sh`

The runner now fails early and clearly if `perl` is missing, instead of failing later while rendering wrk scripts.

## How It Works Internally

### Reuse-port readiness

The lifecycle path now uses a parent/child contract:

1. parent computes a unique temp signal path
2. parent passes it in `SEC4_RT_LASM_READY_FILE`
3. child writes `ready\\n` after runtime bootstrap
4. parent polls file existence and child exit status together
5. parent deletes the signal file once it observes readiness

That keeps readiness deterministic without depending on shared-port probe ambiguity.

### Request hot-path cleanup

The workbench augmentation helper still preserves the public benchmark/operator contract, but it no longer pays JSON parse cost before route dispatch.

The new shape is:

1. apply GET/list defaults
2. branch by exact workbench route
3. parse body only inside a route that needs it
4. synthesize params only for that route

### Debug flag caching

`OnceLock<bool>` ensures the process environment is read once, then reused across the request loop.

## Validation

Commands used in this slice:

```bash
cargo build -p sec4
benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh
bash -n benchmark-suite/scripts/run_workbench_profile.sh
BENCH_DURATION=1s BENCH_WORKBENCH_REQUIRE_WRK2=1 benchmark-suite/scripts/run_workbench_lasm_mode_compare.sh --endpoints wb-tasks-with-comment-tx --lasm-db-adapter postgres
```

Observed outcomes:

- build passed
- public LASM workbench smoke passed
- benchmark runner syntax check passed
- mode-compare remained green on the explicit tx Postgres workload

Short benchmark runs are still noisy, so they should be used as directional checks, not as the only source of truth for scaling claims.

## Inputs, Outputs, Constraints

### Inputs

- `SEC4_RT_LASM_READY_FILE`
- workbench JSON request bodies on canonical public routes
- `SEC4_RT_DEBUG_WORKBENCH_JSON`

### Outputs

- deterministic reuse-port worker readiness
- fewer unnecessary JSON parses on non-workbench or non-body-augmenting requests
- lower per-request debug-flag overhead
- clearer benchmark runner dependency failure

### Constraints

- readiness signaling is process-local and file-based
- it does not change public language/runtime semantics
- benchmark conclusions should still come from repeated sequential runs, not one noisy 1-second sample

## Failure Modes And Diagnostics

If a reuse-port worker never signals readiness, the parent now fails with:

- `LASM cluster worker on port <port> did not become ready within <ms> ms`

If the child exits before signaling readiness, the parent now fails with:

- `LASM cluster worker on port <port> exited early with status ...`

If the benchmark runner lacks `perl`, it now fails early with:

- `perl is required to render benchmark wrk templates`

## Example Usage

Fixed reuse-port cluster run:

```bash
target/debug/sec4 run \
  --path benchmark-suite/services/sec4-lasm-workbench \
  --backend lasm \
  --db-adapter postgres \
  --instances 2
```

Canonical explicit-tx mode compare:

```bash
BENCH_DURATION=1s BENCH_WORKBENCH_REQUIRE_WRK2=1 \
benchmark-suite/scripts/run_workbench_lasm_mode_compare.sh \
  --endpoints wb-tasks-with-comment-tx \
  --lasm-db-adapter postgres
```

## Tradeoffs And Next Steps

This slice improves correctness and removes unnecessary request-path work, but it does not claim to "solve" scaling by itself.

Next:

1. keep scaling/runtime tuning on the canonical Postgres workbench workload
2. use sequential repeated runs for claims, not parallel or shared-state benchmark runs
3. keep closing DB/runtime cleanup exposed by the canonical app before broader architecture work
