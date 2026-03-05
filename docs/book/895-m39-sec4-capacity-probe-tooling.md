# M39 - sec4 Capacity Probe Tooling

## What Was Added

Added reproducible capacity-probe tooling in the benchmark suite:

1. `benchmark-suite/scripts/run_sec4_capacity_probe.sh`
2. `make -C benchmark-suite sec4-capacity-probe` target
3. `benchmark-suite/scripts/test_run_sec4_capacity_probe.sh` dry-run contract test

## Why

We needed a fast way to get concrete scale evidence (request volume + peak RSS) early, instead of debating backend architecture without measurements.

## How It Works

`run_sec4_capacity_probe.sh` does the following:

1. builds sec4 benchmark service (unless `--skip-build`),
2. starts service on a probe port,
3. runs one benchmark profile via `run_profile.sh`,
4. samples service RSS during load,
5. writes deterministic JSON artifact with pass/fail status.

Default pass condition:

1. load run exits successfully,
2. observed request count is greater than or equal to `targetRequests` (default `1000000`).

## Output Artifact

Default output:

- `benchmark-suite/results/summaries/sec4-capacity-probe.json`

Key fields:

1. `pass`
2. `run` (`duration`, `targetRps`, `threads`, `connections`, `targetRequests`)
3. `observed` (`requests`, `requestsPerSec`, `peakRssKb`, `loadGenerator`, `constantRate`, `p99`)
4. `artifacts` (`raw`, `summary`, `serverLog`, `runLog`)

## Notes

When `wrk2` is unavailable, probe uses `wrk` fallback and records `constantRate: false`. This is acceptable for quick local evidence but not for strict constant-rate comparisons.
