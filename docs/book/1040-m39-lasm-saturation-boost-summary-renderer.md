# 1040 M39 Slice: LASM Saturation Boost Summary Renderer

## What It Is

This slice adds a markdown summary renderer for LASM saturation boost tuning artifacts.

Added files:

- `benchmark-suite/scripts/render_lasm_cluster_saturation_boost_summary.sh`
- `benchmark-suite/scripts/test_render_lasm_cluster_saturation_boost_summary.sh`

Updated:

- `benchmark-suite/Makefile` with `lasm-cluster-saturation-boost-summary`

## Why It Exists

By this stage the tuning flow produces multiple JSON artifacts (`matrix`, `analysis`, optional `verify`), but reading them manually is slow during operator handoff.

A deterministic markdown summary gives a quick, auditable view of recommendation outcomes and verification status.

## How It Works Internally

1. Inputs:
   - required: matrix JSON + analysis JSON,
   - optional: recommended-step verification probe JSON.

2. Validation:
   - matrix must contain at least one run,
   - analysis must include numeric `summary.recommendedBoostStep`,
   - recommended step must exist in matrix runs,
   - when verification is provided, `run.autoscaleSaturationBoostStep` must match recommended step.

3. Rendering:
   - emits deterministic markdown sections:
     - metadata and recommendation summary,
     - probe profile,
     - ranked runs table,
     - recommendation row details,
     - optional verification section.

4. Makefile integration:
   - target `lasm-cluster-saturation-boost-summary` renders summary with:
     - matrix + analysis only, or
     - matrix + analysis + verify artifact when `LASM_SATURATION_VERIFY_INPUT` is set.

## Inputs / Outputs and Constraints

Inputs:
- `sec4-lasm-cluster-saturation-boost-matrix.json`
- `sec4-lasm-cluster-saturation-boost-analysis.json`
- optional `sec4-lasm-cluster-capacity-probe-sat-boost-recommended.json`

Output:
- markdown summary (default: `results/summaries/sec4-lasm-cluster-saturation-boost-summary.md`)

Constraints:
- requires `jq`,
- renderer fails fast on artifact-shape drift and recommendation/verification mismatch.

## Failure Modes and Diagnostics

- missing files:
  - `saturation boost matrix file not found: ...`
  - `saturation boost analysis file not found: ...`
  - `recommended verification file not found: ...`
- missing runs:
  - `saturation boost matrix has no runs: ...`
- invalid/missing recommendation:
  - `saturation boost analysis missing summary.recommendedBoostStep: ...`
  - `recommended boost step is not present in matrix runs: ...`
- verification mismatch:
  - `recommended verification boost step mismatch: expected <n>, got <m>`

## Example Usage

Matrix + analysis:

```bash
benchmark-suite/scripts/render_lasm_cluster_saturation_boost_summary.sh \
  benchmark-suite/results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json \
  benchmark-suite/results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json \
  benchmark-suite/results/summaries/sec4-lasm-cluster-saturation-boost-summary.md
```

With verification artifact:

```bash
benchmark-suite/scripts/render_lasm_cluster_saturation_boost_summary.sh \
  benchmark-suite/results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json \
  benchmark-suite/results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json \
  benchmark-suite/results/summaries/sec4-lasm-cluster-saturation-boost-summary.md \
  benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-sat-boost-recommended.json
```

Or via Make:

```bash
make -C benchmark-suite lasm-cluster-saturation-boost-summary \
  LASM_SATURATION_VERIFY_INPUT=results/summaries/sec4-lasm-cluster-capacity-probe-sat-boost-recommended.json
```

## Tradeoffs and Next Steps

Tradeoffs:
- introduces one more artifact type (markdown summary), but reduces operator read overhead.
- summary quality depends on upstream artifact correctness and profile representativeness.

Next steps:
1. feed this summary artifact into benchmark evidence publishing and release handoff notes.
2. continue runtime/proxy hot-path work to improve absolute throughput while preserving deterministic artifact contracts.
