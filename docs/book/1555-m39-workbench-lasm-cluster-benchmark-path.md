# M39: Workbench LASM Cluster Benchmark Path

## What It Is

This slice extends the canonical workbench benchmark runners so `sec4-lasm` can be exercised in real LASM cluster mode on the same DB-backed workload as the single-instance path. The fixed-target matrix, step-load matrix, and full-suite wrapper now accept LASM cluster configuration flags and carry that mode into the emitted run summaries.

It also fixes a step-load harness bug for write endpoints: each step rate now uses its own benchmark run tag, so later rate passes do not collide with earlier request IDs.

## Why It Exists

The roadmap already treats the built-in LASM cluster runtime as a real feature, but the workbench benchmark path only launched single-instance LASM servers. That left the scaling story disconnected from the canonical Postgres app and forced operator tuning back onto synthetic capacity probes.

For the alpha proof path, that is the wrong measurement surface. Scaling work must run against the same transactional app and route contract that users will actually evaluate.

## How It Works Internally

The workbench benchmark scripts now accept and forward these LASM-only flags:

- `--lasm-instances`
- `--lasm-autoscale-max-instances`
- `--lasm-autoscale-target-connections`
- `--lasm-autoscale-check-ms`
- `--lasm-cluster-relay-workers`
- `--lasm-cluster-relay-queue`
- `--lasm-cluster-accept-workers`
- `--lasm-cluster-relay-accept-batch-max`
- `--lasm-cluster-relay-pump-batch-max`

The scripts resolve one of three LASM runtime modes:

- `single`
- `cluster-fixed`
- `cluster-proxy`

The mode is derived from `instances` and `autoscale-max-instances`:

- `instances == 1` => `single`
- `instances > 1 && instances == autoscale-max-instances` => `cluster-fixed`
- `instances > 1 && autoscale-max-instances > instances` => `cluster-proxy`

Relay-only tuning flags are now rejected unless the script is in `cluster-proxy` mode. That prevents meaningless flag combinations from getting all the way to `sec4 run`.

The benchmark and step-matrix JSON rows now include a `lasm` object for `sec4-lasm` runs, carrying:

- DB adapter
- runtime mode
- instance counts
- autoscale controls
- cluster relay tuning overrides

For the step harness, `run_workbench_step_profile.sh` now derives a per-rate tag:

- `base-run-tag-r120`
- `base-run-tag-r300`
- `base-run-tag-r500`

That keeps write IDs unique across the whole step sequence.

## Inputs, Outputs, and Constraints

Inputs:

- existing workbench benchmark matrix
- LASM workbench service path
- real Postgres DSN or sqlite DB base
- optional cluster tuning flags

Outputs:

- the same benchmark artifacts as before
- additional LASM cluster metadata in run summaries
- valid step-load measurements for write endpoints

Constraints:

- single-instance behavior stays the default
- non-LASM implementations are unchanged
- relay-only knobs are intentionally rejected outside proxy-cluster mode

## Failure Modes and Diagnostics

Deterministic script diagnostics now cover:

- cluster relay tuning requested outside proxy-cluster mode
- `--lasm-autoscale-max-instances` requested without `--lasm-instances > 1`
- the underlying `sec4 run` cluster-guard diagnostics if a forwarded value is invalid

The benchmark failure gates from the earlier slice still apply:

- zero completed requests fail
- socket errors fail
- non-2xx/3xx endpoint failures fail

## Example Usage

Fixed-target proxy-cluster run on the canonical Postgres route:

```bash
set -a && source infra/local-postgres/.runtime.env && set +a
BENCH_DURATION=1s BENCH_WORKBENCH_REQUIRE_WRK2=0 \
benchmark-suite/scripts/run_workbench_benchmark_matrix.sh \
  --impls sec4-lasm \
  --endpoints wb-tasks-with-comment \
  --lasm-db-adapter postgres \
  --port 18441 \
  --lasm-instances 2 \
  --lasm-autoscale-max-instances 4 \
  --lasm-cluster-accept-workers 2 \
  --lasm-cluster-relay-pump-batch-max 128
```

Step-load run on the same cluster-backed route:

```bash
set -a && source infra/local-postgres/.runtime.env && set +a
BENCH_STEP_DURATION=1s BENCH_WORKBENCH_REQUIRE_WRK2=0 \
benchmark-suite/scripts/run_workbench_step_matrix.sh \
  --impls sec4-lasm \
  --endpoints wb-tasks-with-comment \
  --lasm-db-adapter postgres \
  --port 18442 \
  --lasm-instances 2 \
  --lasm-autoscale-max-instances 4 \
  --lasm-cluster-accept-workers 2 \
  --lasm-cluster-relay-pump-batch-max 128
```

## Tradeoffs and Next Steps

Tradeoffs:

- the benchmark runner now carries more LASM-specific configuration surface
- cluster tuning is still operator-facing script input, not auto-optimization

Next steps:

- run the same cluster-backed workbench matrix across the remaining endpoints, not only `wb-tasks-with-comment`
- use the resulting artifacts to guide the next runtime tuning pass on the real app
- keep the comparison with `node`, `go`, and `rust` on the same DB-backed contract
