# 1038 M39 Slice: LASM Saturation Boost Matrix Integrated Analysis

## What It Is

This slice upgrades the matrix runner so one command can both:

1. generate the saturation boost matrix, and
2. emit a deterministic recommendation analysis artifact.

Updated files:

- `benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh`
- `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh`
- `benchmark-suite/Makefile`

## Why It Exists

After adding standalone analysis, operators still needed a second command to produce recommendations.

That split increases workflow friction and introduces avoidable operator mistakes (matrix generated but analysis forgotten). Integrated analysis keeps tuning loops deterministic and faster to run.

## How It Works Internally

1. New matrix runner options:
   - `--analysis-out <path>`: controls analysis artifact output path.
   - `--skip-analysis`: disables post-run analysis step explicitly.

2. Default behavior:
   - non-dry-run matrix execution now invokes:
     - `analyze_lasm_cluster_saturation_boost_matrix.sh <matrix> <analysis>`
   - then prints:
     - `recommendedSaturationBoostStep=<n>`

3. Dry-run behavior:
   - plan output now includes:
     - resolved `analysisOut` path,
     - `skipAnalysis=<true|false>`,
     - analysis command plan when analysis is enabled.

4. Makefile wiring:
   - `lasm-cluster-saturation-boost-matrix` now passes both:
     - `--out "$(LASM_SATURATION_MATRIX)"`
     - `--analysis-out "$(LASM_SATURATION_ANALYSIS)"`

## Inputs / Outputs and Constraints

Inputs:
- existing matrix-run inputs (boost steps, load profile, cluster knobs),
- optional analysis artifact path,
- optional skip-analysis mode.

Outputs:
- matrix summary JSON,
- analysis JSON (unless `--skip-analysis`),
- console recommendation line with selected boost step (unless `--skip-analysis`).

Constraints:
- requires both executable scripts when analysis is enabled:
  - matrix probe runner,
  - saturation matrix analyzer.

## Failure Modes and Diagnostics

- missing probe runner:
  - `missing executable probe script: ...`
- missing analyzer when analysis is enabled:
  - `missing executable analysis script: ...`
- existing matrix validation failures remain unchanged.

## Example Usage

Run matrix with integrated analysis:

```bash
benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh \
  --boost-steps 2,4,6,8 \
  --target-requests 1000000 \
  --out benchmark-suite/results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json \
  --analysis-out benchmark-suite/results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json
```

Matrix-only mode:

```bash
benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh \
  --boost-steps 2,4,6,8 \
  --skip-analysis
```

## Tradeoffs and Next Steps

Tradeoffs:
- default integrated behavior adds one extra script call after matrix generation (minimal overhead compared with load probe runtime).
- recommendation quality still depends on representative probe inputs.

Next steps:
1. consume `recommendedSaturationBoostStep` automatically in follow-up verification probes.
2. keep reducing proxy/runtime hot-path overhead toward sustained `>=1M` request evidence.
