# 1028 M39 Slice: LASM Autoscale Cooldown Hysteresis Controls

This slice adds explicit hysteresis controls to LASM cluster autoscaling.

## What changed

1. Added new `sec4 run` LASM flags:
   - `--autoscale-scale-up-cooldown-ms`
   - `--autoscale-scale-down-cooldown-ms`
2. Extended cluster autoscaler loop to track last scale-up and scale-down timestamps.
3. Autoscaler now applies independent cooldown windows before changing worker count in each direction.
4. Added deterministic guard behavior:
   - both values must be `>= 1`,
   - both flags are LASM-only.

## Why

Directly scaling every check interval can cause worker-count oscillation under bursty or short-lived spikes. Cooldown-based hysteresis stabilizes scaling behavior and reduces process churn.

## Validation

- `cargo check -p sec4`
- `cargo test -p sec4 --test commands run_command_rejects_autoscale_scale_up_cooldown_with_c_backend -- --exact`
- `cargo test -p sec4 --test commands run_command_rejects_zero_autoscale_scale_down_cooldown -- --exact`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request -- --exact`
- local proxy probe (`/health` + auth, `wrk -t8 -c256 -d10s`) remained around `~60k req/s` in the current local profile.
