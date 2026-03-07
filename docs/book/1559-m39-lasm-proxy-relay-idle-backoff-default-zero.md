# M39: LASM Proxy Relay Idle Backoff Default Zero

## What it is

This slice lowers the default LASM proxy relay idle-backoff cap from `1` to `0` in `compiler/sec4-cli/src/lasm_cluster_relay_pump.rs`.

## Why it exists

The relay idle-backoff knob was already runtime-tunable, but the default still inserted one extra deferred pump tick after fully idle iterations. On the canonical Postgres-backed transactional workload (`wb-tasks-with-comment`), that default was adding avoidable proxy tail latency without improving throughput or stability.

## How it works internally

`LasmClusterRelayPump::pump_once()` keeps an `idle_backoff_remaining` countdown. When it is non-zero, the relay returns `BackoffDeferred` and skips real socket work for that tick. With the default lowered to `0`, fully idle relays no longer consume an extra defer cycle unless an operator explicitly opts back in through `SEC4_RT_LASM_CLUSTER_RELAY_IDLE_BACKOFF_MAX`.

The runtime override remains intact:

- min: `0`
- max: `32`
- env: `SEC4_RT_LASM_CLUSTER_RELAY_IDLE_BACKOFF_MAX`

Only the default changed.

## Inputs, outputs, and constraints

Inputs:

- canonical LASM workbench app
- Postgres adapter
- proxy-cluster mode

Validated run:

- endpoint: `wb-tasks-with-comment`
- instances: `2`
- autoscale max instances: `4`
- cluster accept workers: `2`
- cluster relay pump batch max: `128`
- load generator: `wrk2`
- target rate: `350 req/s`

Observed output:

- `350.01 req/s`
- `24.30ms` p99
- `0` non-2xx/3xx responses
- `0` socket errors

Artifact:

- `benchmark-suite/results/summaries/sec4-lasm-wb-tasks-with-comment.json`

## Failure modes and diagnostics

If this default is wrong for another topology, operators can still override it explicitly:

```bash
SEC4_RT_LASM_CLUSTER_RELAY_IDLE_BACKOFF_MAX=1
```

Relevant diagnostics remain unchanged:

- benchmark failure gating still fails zero-request runs
- benchmark failure gating still fails socket-error runs
- cluster status continues to report `relayIdleBackoffMax`

## Example usage

Default behavior now needs no extra env:

```bash
source infra/local-postgres/.runtime.env
BENCH_DURATION=1s BENCH_WORKBENCH_REQUIRE_WRK2=0 \
benchmark-suite/scripts/run_workbench_benchmark_matrix.sh \
  --impls sec4-lasm \
  --endpoints wb-tasks-with-comment \
  --port 18630 \
  --lasm-db-adapter postgres \
  --lasm-db-postgres-shared-client-max-active-per-key 8 \
  --lasm-db-postgres-shared-client-max-active-total 12 \
  --lasm-instances 2 \
  --lasm-autoscale-max-instances 4 \
  --lasm-cluster-accept-workers 2 \
  --lasm-cluster-relay-pump-batch-max 128
```

## Tradeoffs and next steps

Tradeoffs:

- lower backoff reduces proxy wake latency
- it may increase idle CPU slightly on some hosts, so the env override stays available

Next steps:

1. keep tuning proxy-cluster tail latency on the same Postgres transactional workload
2. focus next on relay scheduling and dispatch behavior only if measured data still shows a proxy-only penalty
3. avoid broader runtime changes until the canonical workbench app exposes a real bottleneck that survives this lower-latency default
