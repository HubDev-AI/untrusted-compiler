# 1050 M39 Slice: Full-Suite Saturation Combined Preset Targets

## What It Is

This slice adds one-command Make targets that execute both saturation presets
(throughput and latency) in sequence.

Updated files:

- `benchmark-suite/Makefile`
- `benchmark-suite/scripts/test_makefile_profile_targets.sh`
- `benchmark-suite/scripts/test_bench_full_saturation_presets_profile.sh`

## Why It Exists

With separate throughput and latency presets available, exploratory tuning still
required two manual commands for each full sweep.

Combined preset targets simplify the loop into one command while preserving the
existing preset behavior and override surfaces.

## How It Works Internally

1. New targets:
   - `bench-full-saturation-presets`
   - `bench-full-saturation-presets-dry`

2. Delegation:
   - combined run target calls:
     - `bench-full-saturation-throughput`
     - `bench-full-saturation-latency`
   - combined dry-run target calls:
     - `bench-full-saturation-throughput-dry`
     - `bench-full-saturation-latency-dry`

3. Contract coverage:
   - static target-token assertions lock combined target wiring,
   - dynamic dry-run test verifies both preset signatures are present in output
     (`boostSteps=2,4,6,8` and `boostSteps=1,2,3`).

## Inputs / Outputs and Constraints

Inputs:
- no new required inputs; combined targets inherit the existing preset override
  variables.

Outputs:
- two sequential preset runs (or dry-run plans) in one invocation.

Constraints:
- ordering is fixed: throughput preset first, then latency preset.

## Failure Modes and Diagnostics

- target wiring drift triggers `test_makefile_profile_targets.sh` failures,
- missing dual-profile output triggers
  `test_bench_full_saturation_presets_profile.sh` failures with explicit
  missing-profile diagnostics.

## Example Usage

Dry-run both presets:

```bash
make -C benchmark-suite bench-full-saturation-presets-dry
```

Run both presets:

```bash
make -C benchmark-suite bench-full-saturation-presets
```

## Tradeoffs and Next Steps

Tradeoffs:
- combined target increases convenience but also lengthens end-to-end run time
  for non-dry executions.

Next steps:
1. add an optional summary combiner for throughput+latency preset outputs,
2. continue runtime/proxy hot-path optimization and compare results across both
   preset lanes.
