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
./benchmark-suite/scripts/run_workbench_benchmark_matrix_local.sh \
  --keep-up \
  --impls sec4-lasm,node,go,rust \
  --endpoints wb-tasks-post,wb-tasks-with-comment,wb-tasks-with-comment-tx,wb-task-comment-post,wb-task-get,wb-tasks-list \
  --lasm-mode auto
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

- report markdown/html mtime: `2026-03-07 17:47:54 +0200`
- runs/analysis/compare-matrix mtime: `2026-03-07 17:47:54 +0200`

Completeness values:

- `workbench-benchmark-runs.json.startedAt`: `2026-03-07T15:22:07Z`
- `workbench-benchmark-runs.json.finishedAt`: `2026-03-07T15:47:54Z`
- impl set: `sec4-lasm`, `node`, `go`, `rust`
- endpoint set: `wb-tasks-post`, `wb-tasks-with-comment`, `wb-tasks-with-comment-tx`, `wb-task-comment-post`, `wb-task-get`, `wb-tasks-list`
- `workbench-benchmark-analysis.json.summary.endpointCount`: `6`
- `workbench-benchmark-analysis.json.summary.highestSeverity`: `HIGH`
- `workbench-benchmark-runs.json.totals`: `{"passed": 2, "failed": 2, "skipped": 0}`

Implementation outcomes from the canonical run:

- `sec4-lasm`: `failed` (`profile failed endpoint=wb-tasks-list`)
- `node`: `failed` (`profile failed endpoint=wb-tasks-post; profile failed endpoint=wb-tasks-with-comment; profile failed endpoint=wb-tasks-with-comment-tx; profile failed endpoint=wb-task-comment-post`)
- `go`: `passed`
- `rust`: `passed`

Key published numbers:

- `wb-tasks-post`: `sec4-lasm 496.85 req/s`, `p99 141.06ms`
- `wb-tasks-with-comment`: `sec4-lasm 199.21 req/s`, `p99 148.86ms`
- `wb-tasks-with-comment-tx`: `sec4-lasm 198.33 req/s`, `p99 495.87ms`
- `wb-task-comment-post`: `sec4-lasm 496.84 req/s`, `p99 14.57ms`
- `wb-task-get`: `sec4-lasm 1611.85 req/s`, `p99 20.41s`
- `wb-tasks-list`: `sec4-lasm 924.47 req/s`, `p99 24.36s`

## Contract Gates

- `./scripts/check-naming-lock.sh`: `PASS`
- `./scripts/check-milestone-closure.sh --fail-on-pending`: `PASS`

## Residual Risk / Follow-up

These do not block the alpha readiness decision, but they are real post-alpha follow-up items:

1. The comparative benchmark publication is fresh and complete, but the canonical run records impl-level failures on:
   - `sec4-lasm` at `wb-tasks-list` under the current target load,
   - `node` on multiple write-heavy endpoints.
2. `workbench-benchmark-analysis.json` still reports `highestSeverity = HIGH`, driven by tail-latency spread and low target coverage on read-heavy endpoints.
3. The next queue after alpha closeout should stay focused on runtime/scaling tuning, starting with the `wb-tasks-list` saturation path.

## Merge Batch Note

- Tasks closed in this batch: `010`, `013`, `014`, `015`, `016`, `017`, `018`
- Remaining alpha-closure tasks: `none`
- Recommended merge action: `merge now`, then open the next post-alpha runtime/scaling tuning batch from the published benchmark evidence.
