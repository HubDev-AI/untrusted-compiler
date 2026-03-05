# 1043 M39 Slice: Full-Suite Optional Saturation Lane

## What It Is

This slice integrates the LASM saturation bundle into full benchmark-suite orchestration as an optional phase.

Updated files:

- `benchmark-suite/scripts/run_full_benchmark_suite.sh`
- `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
- `benchmark-suite/Makefile`

## Why It Exists

The saturation tooling was available, but running it alongside fixed-target and step-load suites still required separate manual commands.

This slice adds an optional orchestration lane so operators can include saturation evidence in the same full-suite flow when needed.

## How It Works Internally

1. New full-suite flags:
   - `--include-lasm-saturation`
   - `--saturation-skip-verify`
   - `--saturation-boost-steps <csv>`

2. Saturation phase behavior:
   - when enabled, full-suite now runs:
     - `run_lasm_cluster_saturation_boost_bundle.sh`
   - generated saturation summary is passed into final report publish call as optional report input.

3. Guard behavior:
   - `--include-lasm-saturation` requires `sec4-lasm` in `--impls`.
   - invalid usage exits deterministically with:
     - `--include-lasm-saturation requires sec4-lasm in --impls`

4. Makefile integration:
   - added:
     - `bench-full-saturation`
     - `bench-full-saturation-dry`
   - optional verify skip control:
     - `FULL_SATURATION_SKIP_VERIFY=true`

## Inputs / Outputs and Constraints

Inputs:
- existing full-suite impl/endpoint selections,
- optional saturation-lane flags,
- optional saturation boost-step list.

Outputs:
- existing full-suite artifacts,
- plus optional saturation artifacts when lane is enabled:
  - matrix,
  - analysis,
  - optional verify probe,
  - saturation summary,
  - report section populated via summary input.

Constraints:
- saturation lane cannot run unless `sec4-lasm` is included in impl selection.

## Failure Modes and Diagnostics

- invalid saturation lane impl scope:
  - `--include-lasm-saturation requires sec4-lasm in --impls`
- delegated saturation bundle validation errors are surfaced unchanged.

## Example Usage

Dry-run full suite with saturation lane:

```bash
benchmark-suite/scripts/run_full_benchmark_suite.sh \
  --dry-run \
  --impls sec4-lasm \
  --endpoints ping \
  --include-lasm-saturation \
  --saturation-skip-verify
```

Make target:

```bash
make -C benchmark-suite bench-full-saturation \
  IMPLS=sec4-lasm \
  ENDPOINTS=ping \
  FULL_SATURATION_SKIP_VERIFY=true
```

## Tradeoffs and Next Steps

Tradeoffs:
- optional lane keeps default full-suite runtime unchanged, but saturation-enabled runs can take longer.
- lane is intentionally opt-in so standard cross-impl suite usage remains lightweight.

Next steps:
1. add an operator profile preset for saturation-enabled full-suite runs.
2. continue runtime/proxy hot-path optimization and collect refreshed throughput evidence through this integrated flow.
