# 1048 M39 Slice: Full-Suite Saturation Throughput Preset Targets

## What It Is

This slice adds throughput-oriented Make presets for saturation-enabled full
benchmark-suite execution.

Updated files:

- `benchmark-suite/Makefile`
- `benchmark-suite/scripts/test_makefile_profile_targets.sh`
- `benchmark-suite/scripts/test_bench_full_saturation_throughput_profile.sh`

## Why It Exists

Saturation tuning loops were already scriptable, but repeated throughput runs
still required manually retyping a larger set of load and relay overrides.

Preset targets reduce operator friction and keep iterative throughput probes
consistent across runs.

## How It Works Internally

1. New Make targets:
   - `bench-full-saturation-throughput`
   - `bench-full-saturation-throughput-dry`

2. Preset delegation path:
   - each preset target delegates to existing:
     - `bench-full-saturation`
     - `bench-full-saturation-dry`
   - delegation enforces:
     - `IMPLS=sec4-lasm`
     - `ENDPOINTS=ping`
     - `FULL_SATURATION_SKIP_VERIFY=true`

3. Throughput defaults:
   - boost steps: `2,4,6,8`
   - duration: `60s`
   - threads: `12`
   - connections: `512`
   - target requests: `1000000`
   - relay workers: `32`
   - relay queue: `4096`

4. Override surface:
   - all preset knobs can be adjusted via:
     - `LASM_SATURATION_THROUGHPUT_*` Make variables.

5. Contract coverage:
   - static Makefile contract assertions keep preset wiring tokens locked,
   - dynamic dry-run contract test validates preset defaults appear in
     delegated saturation plan output.

## Inputs / Outputs and Constraints

Inputs:
- optional overrides for `LASM_SATURATION_THROUGHPUT_*` variables.

Outputs:
- same full-suite artifacts as existing saturation targets,
- dry-run output with deterministic preset plan values.

Constraints:
- presets are intentionally scoped to `sec4-lasm` + `ping` for throughput
  tuning loops.

## Failure Modes and Diagnostics

- if preset target wiring drifts, `test_makefile_profile_targets.sh` fails with
  missing-token diagnostics,
- if delegated preset values no longer reach saturation plan output,
  `test_bench_full_saturation_throughput_profile.sh` fails with the specific
  missing default (threads/connections/relay/etc.).

## Example Usage

Dry-run throughput preset:

```bash
make -C benchmark-suite bench-full-saturation-throughput-dry
```

Run with one overridden preset knob:

```bash
make -C benchmark-suite bench-full-saturation-throughput \
  LASM_SATURATION_THROUGHPUT_THREADS=16
```

## Tradeoffs and Next Steps

Tradeoffs:
- preset values accelerate repeated tests but are opinionated defaults, not a
  universal optimum.

Next steps:
1. add additional presets (for example latency-first) using the same delegation
   pattern,
2. collect refreshed throughput evidence using these presets to guide
   next hot-path optimization slices.
