# 1047 M39 Slice: Full-Suite Saturation Tuning Knob Forwarding

## What It Is

This slice extends full-suite saturation orchestration so tuning knobs can be
set directly at the full-suite entrypoint and forwarded into the LASM
saturation bundle.

Updated files:

- `benchmark-suite/scripts/run_full_benchmark_suite.sh`
- `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
- `benchmark-suite/Makefile`
- `benchmark-suite/scripts/test_makefile_profile_targets.sh`

## Why It Exists

The optional saturation lane existed, but practical tuning still required
running the bundle script separately when changing load shape or relay sizing.

Forwarding the key knobs through full-suite orchestration keeps iteration
one-command and makes repeated tuning runs deterministic from Make targets.

## How It Works Internally

1. Full-suite saturation flag surface is expanded with pass-through knobs:
   - `--saturation-project-path`
   - `--saturation-duration`
   - `--saturation-threads`
   - `--saturation-connections`
   - `--saturation-target-requests`
   - `--saturation-cluster-relay-workers`
   - `--saturation-cluster-relay-queue`

2. Saturation lane argument forwarding:
   - when `--include-lasm-saturation` is enabled, full-suite builds the bundle
     command and conditionally appends only non-empty pass-through values,
   - forwarded values are delegated to
     `run_lasm_cluster_saturation_boost_bundle.sh`.

3. Makefile integration:
   - `bench-full-saturation` and `bench-full-saturation-dry` now pass
     `LASM_CAPACITY_*` values through full-suite saturation flags,
   - operators can tune boost steps, project path, load shape, request floor,
     and relay queue/workers from a single target invocation.

4. Contract coverage:
   - full-suite dry-run tests assert delegated tuning values and step fanout,
   - Makefile target tests assert saturation forwarding tokens remain wired.

## Inputs / Outputs and Constraints

Inputs:
- existing full-suite selection (`--impls`, `--endpoints`),
- optional saturation lane enablement (`--include-lasm-saturation`),
- optional saturation pass-through tuning flags.

Outputs:
- unchanged full-suite fixed-target/step artifacts,
- optional saturation bundle artifacts with forwarded tuning behavior.

Constraints:
- saturation lane still requires `sec4-lasm` in `--impls`,
- pass-through forwarding is only applied when saturation lane is enabled.

## Failure Modes and Diagnostics

- existing lane guard remains unchanged:
  - `--include-lasm-saturation requires sec4-lasm in --impls`
- invalid pass-through values are surfaced by delegated saturation scripts
  (for example invalid boost-step CSV contracts).

## Example Usage

Dry-run full suite with explicit saturation tuning knobs:

```bash
benchmark-suite/scripts/run_full_benchmark_suite.sh \
  --dry-run \
  --impls sec4-lasm \
  --endpoints ping \
  --include-lasm-saturation \
  --saturation-boost-steps 3,5 \
  --saturation-duration 55s \
  --saturation-threads 3 \
  --saturation-connections 99 \
  --saturation-target-requests 12345 \
  --saturation-cluster-relay-workers 11 \
  --saturation-cluster-relay-queue 222 \
  --saturation-skip-verify
```

Make target with forwarded tuning inputs:

```bash
make -C benchmark-suite bench-full-saturation-dry \
  IMPLS=sec4-lasm \
  ENDPOINTS=ping \
  LASM_CAPACITY_BOOST_STEPS=3,5 \
  LASM_CAPACITY_DURATION=55s \
  LASM_CAPACITY_THREADS=3 \
  LASM_CAPACITY_CONNECTIONS=99 \
  LASM_CAPACITY_TARGET_REQUESTS=12345 \
  LASM_CAPACITY_CLUSTER_RELAY_WORKERS=11 \
  LASM_CAPACITY_CLUSTER_RELAY_QUEUE=222 \
  FULL_SATURATION_SKIP_VERIFY=true
```

## Tradeoffs and Next Steps

Tradeoffs:
- adds more full-suite flags, increasing surface area,
- keeps default behavior unchanged when tuning flags are omitted.

Next steps:
1. add saturation-lane presets for common tuning scenarios (latency-first vs
   throughput-first),
2. use forwarded knobs to run broader cluster tuning experiments and drive the
   remaining 1M req/s gap work.
