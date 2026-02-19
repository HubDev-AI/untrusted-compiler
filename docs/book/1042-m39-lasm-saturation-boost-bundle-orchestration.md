# 1042 M39 Slice: LASM Saturation Boost Bundle Orchestration

## What It Is

This slice adds a single orchestration command for LASM saturation tuning:

- `benchmark-suite/scripts/run_lasm_cluster_saturation_boost_bundle.sh`
- `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_bundle.sh`

It also adds a Makefile entrypoint:

- `make -C benchmark-suite lasm-cluster-saturation-boost-bundle`

## Why It Exists

The tuning workflow previously required several manual steps:

1. run matrix/analysis,
2. optionally run recommended-step verification,
3. render markdown summary.

That split was deterministic but operator-heavy. This slice keeps the same artifact contracts while reducing runbook overhead to one command.

## How It Works Internally

1. Bundle runner delegates to existing scripts:
   - `run_lasm_cluster_saturation_boost_matrix.sh`
   - `render_lasm_cluster_saturation_boost_summary.sh`

2. Default flow:
   - runs matrix with integrated analysis,
   - runs recommended-step verification probe,
   - renders markdown summary including verification results.

3. Optional flow:
   - `--skip-verify` disables recommended-step follow-up probe and renders summary from matrix + analysis only.

4. Dry-run:
   - `--dry-run` prints:
     - bundle-level plan,
     - delegated matrix plan,
     - summary render command plan.

5. Makefile integration:
   - `lasm-cluster-saturation-boost-bundle` forwards canonical matrix/analysis/verify/summary output paths and project/target inputs.

## Inputs / Outputs and Constraints

Inputs:
- same probe/tuning knobs as matrix runner (boost steps, duration, threads, connections, autoscale/relay controls),
- output-path controls for matrix, analysis, verify, and summary artifacts.

Outputs:
- matrix JSON,
- analysis JSON,
- optional recommended-step verify JSON,
- markdown summary.

Constraints:
- depends on executable delegated scripts:
  - `run_lasm_cluster_saturation_boost_matrix.sh`
  - `render_lasm_cluster_saturation_boost_summary.sh`

## Failure Modes and Diagnostics

- missing delegated scripts:
  - `missing executable matrix script: ...`
  - `missing executable summary script: ...`
- invalid boost-step input is surfaced from delegated matrix runner:
  - `boost-steps must contain positive integers, got: <value>`

## Example Usage

Run full bundle:

```bash
benchmark-suite/scripts/run_lasm_cluster_saturation_boost_bundle.sh \
  --boost-steps 2,4,6,8 \
  --target-requests 1000000
```

Run without recommended-step verify:

```bash
benchmark-suite/scripts/run_lasm_cluster_saturation_boost_bundle.sh \
  --boost-steps 2,4,6,8 \
  --skip-verify
```

Or via Make:

```bash
make -C benchmark-suite lasm-cluster-saturation-boost-bundle \
  LASM_CAPACITY_BOOST_STEPS=2,4,6,8 \
  LASM_CAPACITY_TARGET_REQUESTS=1000000
```

## Tradeoffs and Next Steps

Tradeoffs:
- bundle simplifies operator flow but intentionally reuses existing scripts instead of introducing new tuning logic.
- full bundle runtime can be long when verify probe is enabled.

Next steps:
1. add optional publish step integration so bundle can emit a report-ready artifact set automatically.
2. continue runtime/proxy hot-path optimization to move sustained throughput closer to target.
