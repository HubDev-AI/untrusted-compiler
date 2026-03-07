# M39: Workbench LASM Mode Compare Runner

## What It Is

This slice adds a real-workbench LASM mode-compare runner:

- `benchmark-suite/scripts/run_workbench_lasm_mode_compare.sh`

It executes the canonical LASM workbench benchmark in three modes on the same workload:

- `single`
- `cluster-fixed`
- `cluster-proxy`

Then it emits one comparison artifact:

- `benchmark-suite/results/summaries/workbench-lasm-mode-compare.json`

## Why It Exists

After wiring LASM cluster flags into the canonical workbench benchmark runner, the next missing piece was a direct apples-to-apples comparison of runtime modes on the real Postgres app.

Without this runner, scaling decisions still depended on:

- synthetic capacity probes, or
- manual repeated benchmark commands.

That is too weak for the alpha proof path. The operator story needs one deterministic command that answers a simple question:

Which LASM runtime mode is currently best on the real app workload?

## How It Works Internally

The runner wraps:

- `benchmark-suite/scripts/run_workbench_benchmark_matrix.sh`

It runs that benchmark three times for `sec4-lasm` only:

1. `single`
   - `--lasm-instances 1`
   - `--lasm-autoscale-max-instances 1`
2. `fixed`
   - `--lasm-instances N`
   - `--lasm-autoscale-max-instances N`
3. `proxy`
   - `--lasm-instances N`
   - `--lasm-autoscale-max-instances M`
   - optional proxy-only relay tuning flags

After each run, it copies the generated `sec4-lasm` report into a mode-specific report file so later runs do not overwrite the prior evidence.

The final JSON artifact carries:

- per-mode run/report paths
- per-mode aggregate throughput/p99/memory observations
- per-endpoint leaders across modes
- overall mode-win counts
- one `comparison.recommendedMode`

The JSON is also shaped to stay compatible with existing report consumers that already look for:

- `.comparison.recommendedMode`
- `.proxy`
- `.fixed`

## Inputs, Outputs, and Constraints

Inputs:

- benchmark endpoints CSV
- LASM DB adapter selection
- Postgres DSN file or sqlite DB base
- fixed/proxy cluster sizing knobs

Outputs:

- mode-specific run summaries
- mode-specific copied report JSON files
- one combined mode-compare JSON artifact

Constraints:

- proxy mode requires `--autoscale-max-instances > --instances`
- fixed/proxy comparisons require `--instances >= 2`
- proxy-only relay tuning flags are forwarded only to proxy mode

## Failure Modes and Diagnostics

The runner fails deterministically when:

- `--instances < 2`
- `--autoscale-max-instances <= --instances`
- any underlying mode run fails
- the expected `sec4-lasm` report file is missing after a run

Current known limitation:

- proxy-cluster `rssKb` may still be `null` in the canonical workbench benchmark path
- the compare artifact now keeps `peakRssDeltaKb = null` when either side lacks memory evidence instead of inventing a value

## Example Usage

Short real Postgres comparison on the transactional route:

```bash
set -a && source infra/local-postgres/.runtime.env && set +a
BENCH_DURATION=1s BENCH_WORKBENCH_REQUIRE_WRK2=0 \
benchmark-suite/scripts/run_workbench_lasm_mode_compare.sh \
  --endpoints wb-tasks-with-comment \
  --lasm-db-adapter postgres \
  --port 18450 \
  --instances 2 \
  --autoscale-max-instances 4 \
  --cluster-accept-workers 2 \
  --cluster-relay-pump-batch-max 128
```

Observed result from the short local run used to validate this slice:

- `single`: about `348.89 req/s`, p99 about `452.10ms`
- `fixed`: about `348.64 req/s`, p99 about `434.18ms`
- `proxy`: about `349.03 req/s`, p99 about `9.68ms`
- recommended mode: `proxy`

## Tradeoffs and Next Steps

Tradeoffs:

- this adds another benchmark orchestration script to maintain
- current memory comparison remains incomplete until proxy-cluster RSS sampling is wired into this path

Next steps:

- fix reliable RSS capture for proxy-cluster workbench runs
- extend mode comparison beyond one endpoint so the recommendation is based on the full canonical workload
- use the resulting artifact to drive the next LASM runtime tuning pass instead of relying on synthetic probe data
