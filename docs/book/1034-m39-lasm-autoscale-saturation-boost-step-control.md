# 1034 M39 Slice: LASM Autoscale Saturation Boost-Step Control

## What It Is

This slice adds a dedicated LASM autoscale control for relay-saturation-driven scale-up bursts:

- `--autoscale-saturation-boost-step` (default: `4`)

It is independent from normal autoscale scale-up step (`--autoscale-scale-up-step`).

## Why It Exists

Normal scale-up behavior should stay smooth under ordinary load. Saturation events (`relay queue full`) are a stronger pressure signal and may need larger per-check worker jumps than the steady-state scale-up step.

Without a separate saturation boost-step, tuning for fast saturation recovery also forces aggressive normal scaling.

## How It Works Internally

1. CLI/runtime surface:
   - new `run` flag: `--autoscale-saturation-boost-step`.
   - validation:
     - value must be `>= 1`,
     - LASM-only (deterministic C-backend rejection).

2. Autoscale behavior in cluster proxy mode:
   - autoscale loop still computes baseline desired workers from active connections.
   - when relay saturation events are observed in a check window:
     - desired workers are boosted by `autoscale_saturation_boost_step`,
     - per-check scale-up budget for that tick is raised to at least the same boost-step.
   - cooldown and max-instance limits remain enforced.

3. Tooling + contracts:
   - benchmark probe script supports and forwards `--autoscale-saturation-boost-step`.
   - runtime-flag contract + guard scripts now lock this flag field and LASM-only guard.
   - command tests cover zero-value rejection and C-backend rejection.

## Inputs / Outputs and Constraints

Inputs:
- `sec4 run --backend lasm` in cluster proxy mode with optional saturation boost-step override.

Outputs:
- deterministic autoscale behavior with larger burst scale-up potential on saturation ticks.

Constraints:
- this flag does not bypass cooldown windows, max-instance caps, or other autoscale guardrails.

## Failure Modes and Diagnostics

- zero/invalid value:
  - `run failed: --autoscale-saturation-boost-step must be >= 1`
- unsupported backend:
  - `run failed: --autoscale-saturation-boost-step is only supported with --backend lasm`

## Example Usage

```bash
cargo run -p sec4 -- run \
  --path examples/lasm-alpha-full \
  --backend lasm \
  --instances 4 \
  --autoscale-max-instances 8 \
  --autoscale-target-connections 256 \
  --autoscale-scale-up-step 2 \
  --autoscale-saturation-boost-step 6
```

Benchmark probe dry-run:

```bash
benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh \
  --dry-run \
  --autoscale-saturation-boost-step 6
```

## Tradeoffs and Next Steps

Tradeoffs:
- larger saturation burst steps can reduce queue pressure faster, but can increase worker churn if over-tuned.

Next steps:
1. add targeted probe profiles comparing different saturation boost-step values under fixed relay limits,
2. continue hot-path tuning toward the 1M request target while preserving deterministic guard behavior.
