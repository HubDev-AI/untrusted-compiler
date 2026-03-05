# 1039 M39 Slice: LASM Saturation Boost Recommended Follow-Up Verification

## What It Is

This slice adds optional recommended-step follow-up probing to the LASM saturation matrix runner.

Updated files:

- `benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh`
- `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh`
- `benchmark-suite/Makefile`

## Why It Exists

Matrix + analysis identifies a recommended boost step, but operators still had to run a manual confirmatory probe to produce final evidence for that recommendation.

This slice closes that gap by making the follow-up probe one flag away, while preserving explicit opt-in behavior.

## How It Works Internally

1. New matrix-run options:
   - `--verify-recommended`: run one additional capacity probe after analysis.
   - `--verify-out <path>`: output path for that follow-up probe artifact.

2. Guardrails:
   - verification requires analysis output.
   - invalid combination is rejected deterministically:
     - `--verify-recommended --skip-analysis`
     - diagnostic: `verify-recommended requires analysis; remove --skip-analysis`

3. Execution flow (non-dry-run):
   - matrix probes execute for configured boost steps.
   - analysis computes recommended step.
   - runner executes one extra probe with:
     - `--autoscale-saturation-boost-step <recommended>`,
     - `--skip-build`,
     - configured verify output path.
   - emits deterministic trace lines:
     - `verifyingRecommendedBoostStep=<n>`
     - `recommendedVerificationOut=<path>`

4. Dry-run flow:
   - plan now includes:
     - `verifyRecommended=<true|false>`
     - `verifyOut=<path>`
     - placeholder verification command plan when enabled.

5. Makefile integration:
   - new target:
     - `lasm-cluster-saturation-boost-verify`
   - target runs matrix + analysis + recommended follow-up in one command.

## Inputs / Outputs and Constraints

Inputs:
- existing matrix-run knobs and analysis options,
- optional follow-up verification enable/path.

Outputs:
- matrix artifact,
- analysis artifact,
- optional recommended-step follow-up probe artifact.

Constraints:
- follow-up verification depends on successful analysis recommendation output.

## Failure Modes and Diagnostics

- invalid option combination:
  - `verify-recommended requires analysis; remove --skip-analysis`
- existing matrix/analyzer validation failures remain unchanged.

## Example Usage

```bash
benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh \
  --boost-steps 2,4,6,8 \
  --target-requests 1000000 \
  --out benchmark-suite/results/summaries/sec4-lasm-cluster-saturation-boost-matrix.json \
  --analysis-out benchmark-suite/results/summaries/sec4-lasm-cluster-saturation-boost-analysis.json \
  --verify-recommended \
  --verify-out benchmark-suite/results/summaries/sec4-lasm-cluster-capacity-probe-sat-boost-recommended.json
```

Or via Make:

```bash
make -C benchmark-suite lasm-cluster-saturation-boost-verify \
  LASM_CAPACITY_BOOST_STEPS=2,4,6,8 \
  LASM_CAPACITY_TARGET_REQUESTS=1000000
```

## Tradeoffs and Next Steps

Tradeoffs:
- follow-up verification adds one extra probe runtime when enabled.
- recommendation confidence still depends on representative load profile selection.

Next steps:
1. thread follow-up verification artifacts into benchmark evidence publish/report workflows.
2. continue runtime/proxy hot-path optimization to raise sustained throughput beyond current baseline.
