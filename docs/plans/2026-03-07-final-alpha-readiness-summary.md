# Final Alpha Readiness Summary

Date: `2026-03-07`  
Branch/commit: `codex/workbench-canonical-app-01` / `4a8f80f7239e5206be95e880c99b689dd09e17c4`  
Prepared by: `Codex`

## Decision

- Alpha readiness: `READY`
- Reason: no-stub alpha closure evidence is complete: the fresh same-contract DB-backed benchmark publication set exists, naming-lock and milestone-closure checks pass, and the final proof-bundle chain passed on the current tuned baseline.

## Current Closure Checklist

- [x] final same-contract DB-backed benchmark publication artifacts are complete and fresh
- [x] final proof-bundle chain was rerun on the same tuned baseline
- [x] naming-lock and strict milestone-closure checks pass on the final artifact state
- [x] this summary includes exact command sequence, evidence values, and artifact paths

## Release Proof Bundle

Commands run:

```bash
./scripts/run-final-alpha-proof-bundle.sh \
  --artifacts-dir build/release-alpha-gate
```

Results:

- `release-alpha-gate.sh`: `PASS`
- `verify-release-promotion-inputs.sh`: `PASS`
- `generate-release-publish-manifest.sh`: `PASS`
- `verify-release-publish-manifest.sh`: `PASS`

Evidence:

- `build/release-alpha-gate/summary.txt`
- `build/release-alpha-gate/checksums.txt`
- `build/release-alpha-gate/publish-manifest.json`

Evidence freshness:

- `summary.txt` mtime: `2026-03-07 18:37:50 +0200`
- `checksums.txt` mtime: `2026-03-07 18:37:50 +0200`
- `publish-manifest.json` mtime: `2026-03-07 18:37:50 +0200`
- `publish-manifest.generatedAt`: `2026-03-07T16:37:50Z`

Identity hashes:

- `policy identity hash`: `pol_5a4efaf282eee740`
- `compiler identity hash`: `cpl_0_1_0`
- `runtime identity hash`: `rt_1b19b5792b54eba8`

Additional checksums:

- `policy_profile_sha256`: `4a958e469f394b5f46f91392d6876e5af064e0de235a64c73ee979b26f368f93`
- `sec4_binary_sha256`: `d2dddd8d835cfa46a05daacb06b931b8a837b5f8b3f6a4cc76fcc3e7c863886f`
- `runtime_header_sha256`: `8c431d1995077d62323884b9428144c5e33fc0f9c2c3ea179e4518d77468f26e`
- `runtime_source_sha256`: `91bad48621274a7c2f63d64e45aa7d00f7d0100a82d7c37cad1fb2b8e4d97c7e`

## Benchmark Baseline Evidence

Canonical command used for the final publication set:

```bash
BENCH_WORKBENCH_REQUIRE_WRK2=1 \
BENCH_WRK2_BIN=/Users/vladimirtrifonov/src/ai/AILang/benchmark-suite/bin/wrk2 \
BENCH_DURATION=20s \
BENCH_WORKBENCH_PG_DSN='postgres://vladimirtrifonov@127.0.0.1:5432/postgres?sslmode=disable' \
./benchmark-suite/scripts/run_workbench_benchmark_matrix.sh \
  --port 18180 \
  --impls sec4-lasm,node,go,rust \
  --endpoints wb-tasks-post,wb-tasks-with-comment,wb-tasks-with-comment-tx,wb-task-comment-post,wb-task-get,wb-tasks-list \
  --lasm-db-adapter postgres \
  --lasm-mode single
```

Artifacts:

- `benchmark-suite/results/workbench-benchmark-report.md`
- `benchmark-suite/results/workbench-benchmark-report.html`
- `benchmark-suite/results/summaries/workbench-benchmark-runs.json`
- `benchmark-suite/results/summaries/workbench-benchmark-compare-matrix.json`
- `benchmark-suite/results/summaries/workbench-benchmark-analysis.json`

Canonical-family note:

- The closeout benchmark family is `workbench-benchmark-*`.
- `workbench-full-benchmark-*` outputs remain tuning/exploration artifacts and are not used as the alpha closeout publication set because they only cover a narrower tuning slice and still record failed suite phases.

Freshness:

- report markdown/html mtime: `2026-03-07 20:38:37 +0200`
- runs/analysis/compare-matrix mtime: `2026-03-07 20:38:37 +0200`

Completeness values:

- `workbench-benchmark-runs.json.startedAt`: `2026-03-07T18:29:46Z`
- `workbench-benchmark-runs.json.finishedAt`: `2026-03-07T18:38:37Z`
- impl set: `sec4-lasm`, `node`, `go`, `rust`
- endpoint set: `wb-tasks-post`, `wb-tasks-with-comment`, `wb-tasks-with-comment-tx`, `wb-task-comment-post`, `wb-task-get`, `wb-tasks-list`
- `workbench-benchmark-analysis.json.summary.endpointCount`: `6`
- `workbench-benchmark-analysis.json.summary.highestSeverity`: `HIGH`
- `workbench-benchmark-runs.json.totals`: `{"passed": 4, "failed": 0, "skipped": 0}`

Implementation outcomes from the canonical run:

- `sec4-lasm`: `passed`
- `node`: `passed`
- `go`: `passed`
- `rust`: `passed`

Key published numbers:

- `wb-tasks-post`: `sec4-lasm 491.81 req/s`, `go 391.34 req/s`, `rust 41.78 req/s`, `node 35.90 req/s`
- `wb-tasks-with-comment`: `sec4-lasm 198.00 req/s`, `go 95.82 req/s`, `rust 40.74 req/s`, `node 39.88 req/s`
- `wb-tasks-with-comment-tx`: `sec4-lasm 198.13 req/s`, `go 90.00 req/s`, `rust 40.51 req/s`, `node 38.44 req/s`
- `wb-task-comment-post`: `sec4-lasm 491.79 req/s`, `go 297.55 req/s`, `rust 43.31 req/s`, `node 42.57 req/s`
- `wb-task-get`: `sec4-lasm 2465.50 req/s`, `go 212.20 req/s`, `rust 42.24 req/s`, `node 28.81 req/s`
- `wb-tasks-list`: `sec4-lasm 1458.07 req/s`, `go 139.46 req/s`, `rust 21.66 req/s`, `node 9.53 req/s`

## Contract Gates

- `./scripts/check-naming-lock.sh`: `PASS`
- `./scripts/check-milestone-closure.sh --fail-on-pending`: `PASS`

## Residual Risk / Follow-up

These do not block the alpha readiness decision, but they are real post-alpha follow-up items:

1. `workbench-benchmark-analysis.json` still reports `highestSeverity = HIGH`, driven by large cross-runtime tail-latency spread, not by sec4 functional failures.
2. The mixed-workload publication run is stable in LASM `single` mode; cluster/fixed/proxy remain post-alpha tuning space, not alpha-proof defaults.
3. The next queue after alpha closeout should stay focused on runtime/scaling tuning from this clean publication baseline.

## Merge Batch Note

- Tasks closed in this batch: `010`, `013`, `014`, `015`, `016`, `017`, `018`
- Remaining alpha-closure tasks: `none`
- Recommended merge action: `merge now`, then open the next post-alpha runtime/scaling tuning batch from the published benchmark evidence.
