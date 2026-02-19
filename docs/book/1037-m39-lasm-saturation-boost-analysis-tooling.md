# 1037 M39 Slice: LASM Saturation Boost Analysis Tooling

## What It Is

This slice adds deterministic analysis tooling for LASM saturation boost matrix artifacts:

- `benchmark-suite/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh`
- `benchmark-suite/scripts/test_analyze_lasm_cluster_saturation_boost_matrix.sh`

It also wires a Makefile target:

- `make -C benchmark-suite lasm-cluster-saturation-boost-analyze`

## Why It Exists

The saturation matrix runner produces comparable per-step results, but operators still needed manual inspection to choose a recommended boost step.

Manual selection is slow and inconsistent. This analysis pass makes the recommendation deterministic and auditable from the artifact itself.

## How It Works Internally

1. Input validation:
   - requires a non-empty `.runs[]` matrix.
   - validates required run fields and numeric ranges:
     - `saturationBoostStep >= 1`,
     - `requests >= 0`,
     - `requestsPerSec >= 0`,
     - `peakRssKb >= 0`,
     - `pass` boolean.

2. Ranking model:
   - sorts runs deterministically by:
     - pass first (`pass=true` ahead of failures),
     - higher `requestsPerSec`,
     - higher `requests`,
     - lower `peakRssKb`,
     - lower `saturationBoostStep`.

3. Output artifact:
   - writes analysis JSON with:
     - `summary` (run/pass/fail counts, selection mode, recommended step),
     - `recommended` run row,
     - `rankedRuns` full ordered list,
     - original matrix metadata (`impl`, `run`, `boostSteps`).

4. Selection mode:
   - if at least one run passed target requests, mode is `pass-first`,
   - otherwise mode is `throughput-best-no-pass`.

## Inputs / Outputs and Constraints

Inputs:
- matrix JSON from `run_lasm_cluster_saturation_boost_matrix.sh`.

Outputs:
- deterministic analysis JSON (for example `results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json`).

Constraints:
- requires `jq`,
- matrix rows must satisfy required typed fields; malformed rows fail fast.

## Failure Modes and Diagnostics

- missing matrix file:
  - `saturation boost matrix file not found: <path>`
- empty or missing runs:
  - `matrix runs must be a non-empty array: <path>`
- malformed row fields:
  - `matrix runs contain invalid fields: <path>`

## Example Usage

```bash
benchmark-suite/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh \
  benchmark-suite/results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json \
  benchmark-suite/results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json
```

Or through Make:

```bash
make -C benchmark-suite lasm-cluster-saturation-boost-analyze \
  LASM_SATURATION_MATRIX=results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json \
  LASM_SATURATION_ANALYSIS=results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json
```

## Tradeoffs and Next Steps

Tradeoffs:
- this is selection tooling, not a runtime hot-path optimization by itself.
- recommendation quality depends on the measured probe profile.

Next steps:
1. feed analysis recommendations into automated follow-up probe presets.
2. continue proxy/runtime hot-path tuning toward sustained `>=1M` total-request evidence.
