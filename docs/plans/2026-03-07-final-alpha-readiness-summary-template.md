# Final Alpha Readiness Summary (Template)

Date: `<YYYY-MM-DD>`  
Branch/commit: `<branch>` / `<commit>`
Prepared by: `<name>`

## Guardrail

Do not set `Alpha readiness: READY` until every checklist item in:

1. **Benchmark Baseline Evidence (Tuned)** and
2. **Release Proof Bundle**

is complete on the same tuned baseline.

## Decision

- Alpha readiness: `<READY|NOT READY>`
- Reason: `<one-sentence decision rationale>`

## Current Closure Checklist

- [ ] final same-contract DB-backed benchmark publication artifacts are complete and fresh
- [ ] final proof-bundle chain was rerun on the same tuned baseline
- [ ] naming-lock and strict milestone-closure checks pass on the final artifact state
- [ ] this summary includes exact command sequence, timestamps, and artifact paths

If any item is unchecked, decision must remain `NOT READY`.

## Release Proof Bundle

Commands run:

```bash
# from repo root
./scripts/run-final-alpha-proof-bundle.sh \
  --artifacts-dir build/release-alpha-gate
```

Command timestamp window:

- started: `<UTC timestamp>`
- finished: `<UTC timestamp>`

Results:

- `release-alpha-gate.sh`: `<PASS|FAIL>`
- `verify-release-promotion-inputs.sh`: `<PASS|FAIL>`
- `generate-release-publish-manifest.sh`: `<PASS|FAIL>`
- `verify-release-publish-manifest.sh`: `<PASS|FAIL>`

Evidence:

- `build/release-alpha-gate/summary.txt`
- `build/release-alpha-gate/checksums.txt`
- `build/release-alpha-gate/publish-manifest.json`

Evidence freshness check:

- `summary.txt` mtime: `<timestamp>`
- `checksums.txt` mtime: `<timestamp>`
- `publish-manifest.json` mtime: `<timestamp>`
- `publish-manifest.generatedAt`: `<timestamp>`

Identity hashes:

- `policy identity hash`: `<value>`
- `compiler identity hash`: `<value>`
- `runtime identity hash`: `<value>`

## Benchmark Baseline Evidence (Tuned)

Command sequence used:

```bash
# canonical full-suite local Postgres run
benchmark-suite/scripts/run_workbench_full_benchmark_suite_local.sh \
  --keep-up \
  --endpoints wb-tasks-post,wb-tasks-with-comment,wb-tasks-with-comment-tx,wb-task-comment-post,wb-task-get,wb-tasks-list \
  --lasm-mode auto \
  --lasm-mode-compare-repeats 3
```

Artifacts:

- `benchmark-suite/results/workbench-benchmark-report.md`
- `benchmark-suite/results/workbench-benchmark-report.html`
- `benchmark-suite/results/workbench-full-benchmark-report.md`
- `benchmark-suite/results/workbench-full-benchmark-report.html`
- `benchmark-suite/results/summaries/workbench-benchmark-runs.json`
- `benchmark-suite/results/summaries/workbench-benchmark-compare-matrix.json`
- `benchmark-suite/results/summaries/workbench-benchmark-analysis.json`
- `benchmark-suite/results/summaries/workbench-full-runs.json`

Required completeness fields (copy exact values):

- `workbench-benchmark-runs.json`:
  - `startedAt`: `<value>`
  - `finishedAt`: `<value>`
  - `runs[].impl`: `<list>`
  - `totals`: `<json>`
- `workbench-benchmark-compare-matrix.json`:
  - `endpoints | length`: `<n>`
  - `endpoints[].compared[].impl` unique set: `<list>`
- `workbench-benchmark-analysis.json`:
  - `summary.endpointCount`: `<n>`
  - `summary.highestSeverity`: `<value>`
- `workbench-full-runs.json`:
  - `suiteResult`: `<value>`
  - `phaseExitCodes`: `<json>`

Expected contract for final closeout evidence:

- impl set includes: `sec4-lasm`, `node`, `go`, `rust`
- endpoint set includes all 6 canonical workbench endpoints
- artifact timestamps are from this final closeout run window

Key numbers:

- `<endpoint 1>`: `<rps / p99 / coverage>`
- `<endpoint 2>`: `<rps / p99 / coverage>`
- `<endpoint 3>`: `<rps / p99 / coverage>`

## Contract Gates

- `scripts/check-naming-lock.sh`: `<PASS|FAIL>`
- `scripts/check-milestone-closure.sh --fail-on-pending`: `<PASS|FAIL>`

Gate run timestamps:

- naming lock: `<timestamp>`
- milestone closure: `<timestamp>`

## Blockers / Residual Risk

- `<none>` or `<exact unresolved blocker(s)>`

## Merge Batch Note

- Tasks closed in this batch: `<ids>`
- Remaining tasks: `<ids or none>`
- Recommended merge action: `<merge now | wait for blocker>`

## Missing Inputs Tracker (Use Until Ready)

- [ ] Fresh complete benchmark matrix artifacts (cross-impl, same-contract, 6 endpoints)
- [ ] Fresh proof-bundle artifacts under `build/release-alpha-gate/`
- [ ] Final summary filled with exact evidence values above
