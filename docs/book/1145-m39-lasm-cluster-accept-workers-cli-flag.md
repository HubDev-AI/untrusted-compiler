# 1145 M39 Slice: LASM Cluster Accept Workers CLI Flag

This slice exposes accept-worker tuning as a first-class `sec4 run` flag.

## What changed

1. Added `--cluster-accept-workers <n>` to `sec4 run`.
2. Wired the flag through run-command dispatch into LASM cluster config.
3. Updated accept-worker resolution to prioritize explicit CLI value, then env (`SEC4_RT_LASM_CLUSTER_ACCEPT_WORKERS`), then default.
4. Added deterministic guardrails:
   - value must be `>= 1`,
   - LASM backend only,
   - cluster mode only,
   - rejected in fixed reuse-port cluster mode (where proxy relay is not used).

## Why

Accept-worker pool sizing was previously env-only. Exposing the control directly on `sec4 run` makes cluster tuning reproducible in scripts/CI/operator workflows without relying on ambient environment state.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
