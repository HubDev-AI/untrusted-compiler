# 1182 M39 Slice: Run Flags for Cluster Backend Connect Timing

This slice exposes cluster backend-connect timing knobs directly on `sec4 run`.

## What changed

1. Added two new run flags:
   - `--cluster-backend-connect-timeout-ms <n>`
   - `--cluster-backend-connect-cooldown-ms <n>`
2. Added deterministic guard diagnostics for both flags:
   - lower bound `>= 1`,
   - LASM-only usage,
   - cluster-mode-only usage (`--instances > 1`),
   - fixed reuse-port cluster mode rejection (flags are not used there).
3. Updated runtime resolution ordering:
   - explicit CLI override first,
   - `SEC4_RT_LASM_CLUSTER_BACKEND_CONNECT_TIMEOUT_MS` / `SEC4_RT_LASM_CLUSTER_BACKEND_CONNECT_COOLDOWN_MS` next,
   - defaults last.

## Why

These flags make cluster failure-retry/connect tuning reproducible from command invocations and benchmark scripts, without relying on environment-only configuration.

## Validation

1. `cargo test -p sec4 --test commands run_command_rejects_cluster_backend_connect_timeout_with_c_backend`
2. `cargo test -p sec4 --test commands run_command_rejects_zero_cluster_backend_connect_timeout`
3. `cargo test -p sec4 --test commands run_command_rejects_cluster_backend_connect_timeout_without_cluster_mode`
4. `cargo test -p sec4 --test commands run_command_rejects_cluster_backend_connect_cooldown_with_c_backend`
5. `cargo test -p sec4 --test commands run_command_rejects_zero_cluster_backend_connect_cooldown`
6. `cargo test -p sec4 --test commands run_command_rejects_cluster_backend_connect_cooldown_without_cluster_mode`
