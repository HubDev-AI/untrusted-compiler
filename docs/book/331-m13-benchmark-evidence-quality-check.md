# 331 M13 Slice: Benchmark Evidence Quality Check

This chapter documents a quality checker for benchmark evidence artifacts.

## What it is

Added:
- `scripts/check-benchmark-evidence-quality.sh`
- `scripts/test-check-benchmark-evidence-quality.sh`
- `scripts/test-benchmark-workflow-quality-gates.sh`
- `.github/workflows/benchmark-smoke.yml`
- `.github/workflows/benchmark-trend.yml` quality-gate step (`--fail-on-warning`)
- `.github/workflows/benchmark-cross-impl-evidence.yml` quality-gate step (`--fail-on-warning`)

## Why it exists

Milestone closure currently checks evidence presence and shape (`compare-matrix` + trend entry), but not evidence quality posture.

After adding `wrk` fallback support, local runs can be non-constant-rate. Those runs are still useful for smoke confidence, but should be explicitly marked as lower-quality for threshold interpretation.

## How it works internally

`check-benchmark-evidence-quality.sh` reads compare-matrix leaders and checks:

1. endpoint contract integrity:
   - non-empty `compared` rows,
   - leader endpoint matches endpoint group,
   - leader row is present in `compared`,
2. leader `p99` exists and parses to a numeric value greater than `0`,
3. leader `constantRate` posture:
   - `true` -> PASS
   - `false` -> WARN (`non-constant-rate run`)

Modes:
- default: prints PASS/WARN summary and exits `0` even with warnings,
- `--fail-on-warning`: exits non-zero on warnings for stricter gates.
- malformed endpoint contract checks are `FAIL` and always exit non-zero (`2`).

## Tests

`test-check-benchmark-evidence-quality.sh` validates:
- PASS on canonical cross-impl sample matrix,
- WARN on non-constant-rate sample matrix,
- non-zero exit when `--fail-on-warning` is used with warning matrix,
- hard FAIL when leader is not present in compared rows.

Benchmark smoke CI now runs this test.
Benchmark smoke CI also runs `test-benchmark-workflow-quality-gates.sh` to enforce strict-quality gate presence in live benchmark workflows.

## Inputs, outputs, and constraints

- Input:
  - compare matrix JSON (default: `benchmark-suite/results/summaries/compare-matrix.json`)
- Output:
  - endpoint-level PASS/WARN table and overall quality status.
- Constraints:
  - this checker is currently advisory by default and does not replace strict closure gates.

## Example usage

```bash
scripts/check-benchmark-evidence-quality.sh
scripts/check-benchmark-evidence-quality.sh --fail-on-warning
```

## Tradeoffs and next steps

- Tradeoff:
  - advisory mode remains useful for local fallback workflows, but scheduled trend CI and cross-impl evidence CI now run strict mode and fail on WARN quality posture.
- Next:
  - evaluate promoting strict quality mode into alpha release promotion workflows once closure-refresh artifact ingestion is fully automated in CI.
