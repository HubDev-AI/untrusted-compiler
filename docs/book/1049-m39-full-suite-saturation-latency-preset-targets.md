# 1049 M39 Slice: Full-Suite Saturation Latency Preset Targets

## What It Is

This slice adds latency-oriented Make presets for saturation-enabled full-suite
benchmark runs.

Updated files:

- `benchmark-suite/Makefile`
- `benchmark-suite/scripts/test_makefile_profile_targets.sh`
- `benchmark-suite/scripts/test_bench_full_saturation_latency_profile.sh`

## Why It Exists

Throughput presets cover high-load tuning loops, but operators also need a
quick path for lower-load latency-focused sweeps without manually restating
every knob.

Latency presets provide a deterministic low-intensity baseline while preserving
the same saturation lane orchestration path.

## How It Works Internally

1. New Make targets:
   - `bench-full-saturation-latency`
   - `bench-full-saturation-latency-dry`

2. Delegation:
   - presets delegate into:
     - `bench-full-saturation`
     - `bench-full-saturation-dry`
   - enforce:
     - `IMPLS=sec4-lasm`
     - `ENDPOINTS=ping`
     - `FULL_SATURATION_SKIP_VERIFY=true`

3. Latency defaults:
   - boost steps: `1,2,3`
   - duration: `30s`
   - threads: `4`
   - connections: `64`
   - target requests: `250000`
   - relay workers: `16`
   - relay queue: `2048`

4. Override surface:
   - defaults are tunable via:
     - `LASM_SATURATION_LATENCY_*` variables.

5. Contract coverage:
   - static Makefile token checks ensure preset wiring stays intact,
   - dynamic dry-run contract test verifies delegated latency defaults appear in
     saturation plan output.

## Inputs / Outputs and Constraints

Inputs:
- optional `LASM_SATURATION_LATENCY_*` overrides.

Outputs:
- full-suite artifacts using the saturation lane under latency-oriented default
  settings.

Constraints:
- preset scope remains `sec4-lasm` + `ping`,
- verify probe is skipped by default for faster iterative loops.

## Failure Modes and Diagnostics

- wiring drift in Make targets triggers `test_makefile_profile_targets.sh`
  failures,
- delegated value propagation drift triggers
  `test_bench_full_saturation_latency_profile.sh` failures with specific missing
  defaults.

## Example Usage

Dry-run latency preset:

```bash
make -C benchmark-suite bench-full-saturation-latency-dry
```

Run with one overridden latency knob:

```bash
make -C benchmark-suite bench-full-saturation-latency \
  LASM_SATURATION_LATENCY_CONNECTIONS=96
```

## Tradeoffs and Next Steps

Tradeoffs:
- preset values simplify iteration but are intentionally opinionated and not
  globally optimal.

Next steps:
1. collect side-by-side throughput vs latency preset evidence to identify
   low-latency autoscale/relay tradeoffs,
2. continue proxy/runtime hot-path optimization against the 1M req/s target.
