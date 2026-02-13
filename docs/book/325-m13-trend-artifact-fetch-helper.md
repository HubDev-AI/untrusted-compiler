# 325 M13 Slice: Trend Artifact Fetch Helper

This chapter documents the helper that pulls the latest successful benchmark-trend artifact package from GitHub Actions.

## What it is

Updated:
- `benchmark-suite/scripts/fetch_trend_artifact.sh`
- `benchmark-suite/scripts/test_fetch_trend_artifact.sh`
- `.github/workflows/benchmark-smoke.yml`
- `docs/book/322-m13-first-trend-run-results-note.md`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

The remaining M13-S2 task requires live trend metrics from CI artifacts. This helper standardizes artifact retrieval so operators can fetch the right package quickly and consistently.

## How it works internally

`fetch_trend_artifact.sh`:
1. resolves repo slug (from `--repo` or `origin` remote),
2. queries latest successful run for workflow `benchmark-trend.yml`,
3. downloads artifact `benchmark-trend-node-ping-decode` into a target directory.

A `--dry-run` mode prints the exact `gh` commands without network calls.

## Tests

`test_fetch_trend_artifact.sh` validates dry-run output includes:
- `gh run list ...`,
- `gh run download <run_id> ...`.

Benchmark smoke CI now runs this test.

## Inputs, outputs, and constraints

- Inputs:
  - GitHub repo slug and workflow/artifact names (defaults provided),
  - authenticated `gh` session for live mode.
- Output:
  - downloaded artifact directory containing benchmark trend results.
- Constraints:
  - live fetch requires valid GitHub auth token and network access.

## Example usage

```bash
benchmark-suite/scripts/fetch_trend_artifact.sh \
  --repo HubDev-AI/untrusted-compiler \
  --out-dir benchmark-suite/results/trend-download
```

## Tradeoffs and next steps

- Tradeoff:
  - helper depends on GitHub CLI auth state and cannot run unauthenticated.
- Next:
  - chain fetch + import in one convenience command once live artifact cadence is stable.
