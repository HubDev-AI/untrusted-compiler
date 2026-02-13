# 331 M13 Slice: Benchmark Evidence Quality Check

This chapter documents a quality checker for benchmark evidence artifacts.

## What it is

Added:
- `scripts/check-benchmark-evidence-quality.sh`
- `scripts/test-check-benchmark-evidence-quality.sh`
- `.github/workflows/benchmark-smoke.yml`

## Why it exists

Milestone closure currently checks evidence presence and shape (`compare-matrix` + trend entry), but not evidence quality posture.

After adding `wrk` fallback support, local runs can be non-constant-rate. Those runs are still useful for smoke confidence, but should be explicitly marked as lower-quality for threshold interpretation.

## How it works internally

`check-benchmark-evidence-quality.sh` reads compare-matrix leaders and checks:

1. leader `p99` exists and parses to a numeric value greater than `0`,
2. leader `constantRate` posture:
   - `true` -> PASS
   - `false` -> WARN (`non-constant-rate run`)

Modes:
- default: prints PASS/WARN summary and exits `0` even with warnings,
- `--fail-on-warning`: exits non-zero on warnings for stricter gates.

## Tests

`test-check-benchmark-evidence-quality.sh` validates:
- PASS on canonical cross-impl sample matrix,
- WARN on non-constant-rate sample matrix,
- non-zero exit when `--fail-on-warning` is used with warning matrix.

Benchmark smoke CI now runs this test.

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
  - advisory mode avoids breaking local fallback workflows but allows WARN quality posture.
- Next:
  - decide whether promotion gates should require `--fail-on-warning` in production-grade benchmark evidence workflows.
