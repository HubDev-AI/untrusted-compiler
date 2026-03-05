# 1181 M39 Slice: Run Flag for Cluster Relay Accept Batch Size

This slice exposes relay accept-batch tuning directly on the `sec4 run` command.

## What changed

1. Added `--cluster-relay-accept-batch-max <n>` to `sec4 run`.
2. Added deterministic guard diagnostics for this flag:
   - `>= 1` lower-bound validation,
   - LASM-only usage validation,
   - cluster-mode-only validation (`--instances > 1`),
   - fixed reuse-port cluster mode rejection (flag not used in that mode).
3. Updated relay accept-batch runtime resolution to use:
   - explicit CLI override first,
   - `SEC4_RT_LASM_CLUSTER_RELAY_ACCEPT_BATCH_MAX` next,
   - default value last.

## Why

Direct CLI control makes cluster hot-path tuning faster and reproducible during load/perf work, without requiring environment-variable-only workflows.

## Validation

1. `cargo test -p sec4 --test commands run_command_rejects_cluster_relay_accept_batch_max_with_c_backend`
2. `cargo test -p sec4 --test commands run_command_rejects_zero_cluster_relay_accept_batch_max`
3. `cargo test -p sec4 --test commands run_command_rejects_cluster_relay_accept_batch_max_without_cluster_mode`
