# 1035 M39 Slice: LASM Saturation Boost Matrix Tooling

## What It Is

This slice adds a matrix runner for LASM cluster capacity probes across multiple saturation boost-step values:

- `benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh`
- `benchmark-suite/scripts/test_run_lasm_cluster_saturation_boost_matrix.sh`

It also wires a Makefile target:

- `make -C benchmark-suite lasm-cluster-saturation-boost-matrix`

## Why It Exists

After introducing `--autoscale-saturation-boost-step`, tuning requires repeatable comparison across several step values under the same load profile.

Running each probe manually is slow and error-prone. A single matrix command keeps the workflow deterministic and auditable.

## How It Works Internally

1. Matrix inputs:
   - accepts `--boost-steps <csv>` (default `2,4,6`) and forwards shared probe settings to each run.

2. Execution model:
   - for each boost step, runs `run_lasm_cluster_capacity_probe.sh` with:
     - `--autoscale-saturation-boost-step <step>`
     - deterministic per-step output file:
       - `sec4-lasm-cluster-capacity-probe-sat-boost-<step>.json`
   - non-dry-run mode auto-enables `--skip-build` after first probe iteration to avoid redundant rebuilds.

3. Output contracts:
   - dry-run prints the full per-step execution plan,
   - run mode writes matrix summary JSON with:
     - selected boost steps,
     - per-step pass/fail + requests + requests/sec + peak RSS,
     - referenced per-step summary paths.

4. Tooling integration:
   - Makefile target `lasm-cluster-saturation-boost-matrix` forwards:
     - `LASM_CAPACITY_PROJECT_PATH`
     - `LASM_CAPACITY_TARGET_REQUESTS`
     - `LASM_CAPACITY_BOOST_STEPS`

## Inputs / Outputs and Constraints

Inputs:
- same probe-family settings as cluster capacity probe (`duration`, `threads`, `connections`, autoscale/relay knobs),
- CSV list of positive boost-step integers.

Outputs:
- per-step probe summaries,
- one matrix summary artifact.

Constraints:
- requires the existing capacity probe script + dependencies (`wrk`, `jq`, etc.),
- validates boost-step input shape before execution.

## Failure Modes and Diagnostics

- invalid boost-step token:
  - `boost-steps must contain positive integers, got: <value>`
- invalid range:
  - `boost-steps must be >= 1, got: 0`
- missing probe runner:
  - `missing executable probe script: .../run_lasm_cluster_capacity_probe.sh`

## Example Usage

Dry-run:

```bash
benchmark-suite/scripts/run_lasm_cluster_saturation_boost_matrix.sh \
  --dry-run \
  --boost-steps 2,4,7
```

Execute:

```bash
make -C benchmark-suite lasm-cluster-saturation-boost-matrix \
  LASM_CAPACITY_BOOST_STEPS=2,4,7 \
  LASM_CAPACITY_TARGET_REQUESTS=500000
```

## Tradeoffs and Next Steps

Tradeoffs:
- matrix runs can be time-consuming because each boost-step executes a full probe profile.

Next steps:
1. add optional best-step selector output (for example max requests/sec under pass=true),
2. continue runtime hot-path tuning while using matrix artifacts as deterministic before/after evidence.
