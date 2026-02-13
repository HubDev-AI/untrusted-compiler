# 225 M10 Slice: Matrix Tail-Latency and Target-Coverage Analysis

This chapter documents deterministic analysis over benchmark comparison matrices.

## What it is

Added:
- `benchmark-suite/scripts/analyze_matrix.sh`
- `benchmark-suite/scripts/test_analyze_matrix.sh`
- Make target:
  - `make -C benchmark-suite analyze-matrix`

The analysis script produces a JSON artifact with per-endpoint risk signals and an aggregate severity summary.

## Why it exists

M10 exit criteria require explicit tail-latency and failure-mode analysis, not only raw throughput rankings. This slice turns matrix data into deterministic findings that can gate reports/CI.

## How it works internally

1. Reads `compare-matrix.json` endpoint groups.
2. Parses numeric p99 values from latency strings.
3. Computes per-endpoint metrics:
   - p99 min/max,
   - p99 spread ratio,
   - leader target-coverage percentage.
4. Emits findings using stable rules:
   - `P99_SPREAD_MEDIUM` for spread >= 1.5x,
   - `P99_SPREAD_HIGH` for spread >= 2.5x,
   - `LEADER_TARGET_COVERAGE_WARN` for leader coverage < 95%,
   - `LEADER_TARGET_COVERAGE_LOW` for leader coverage < 90%,
   - `ZERO_THROUGHPUT_IMPLEMENTATION` if any impl reports zero throughput.
5. Aggregates a summary with finding counts and highest severity.

## Inputs, outputs, and constraints

- Input:
  - `compare-matrix.json` from comparison scripts.
- Output:
  - analysis JSON (`results/summaries/analysis.json` by default).
- Constraints:
  - p99 parsing expects numeric content in p99 strings,
  - analysis is threshold-based and deterministic (no statistical inference).

## Failure modes and diagnostics

- missing matrix file -> explicit error.
- matrix without endpoints -> explicit error.
- malformed matrix JSON -> `jq` parse failure.

## Example usage

```bash
make -C benchmark-suite compare-matrix
make -C benchmark-suite analyze-matrix
```

Direct invocation:

```bash
benchmark-suite/scripts/analyze_matrix.sh \
  benchmark-suite/results/summaries/compare-matrix.json \
  benchmark-suite/results/summaries/analysis.json
```

## Tradeoffs and next steps

- Tradeoff:
  - current rules are static thresholds; they do not account for hardware-specific baseline variance.
- Next:
  - allow policy-configurable analysis thresholds,
  - include error-rate and resource metrics once they are present in report bundles,
  - integrate analysis summary into published benchmark markdown.
