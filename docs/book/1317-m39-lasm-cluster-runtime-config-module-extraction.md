# M39: LASM Cluster Runtime Config Module Extraction

Date: 2026-02-21  
Milestone: M39-S2B (LASM runtime load hardening)

## What Changed

- Added module file `/Users/vladimirtrifonov/src/ai/AILang/compiler/sec4-cli/src/lasm_cluster_runtime_config.rs`.
- Moved LASM cluster runtime-config helper surface from `main.rs`, including:
  - proxy sizing helpers (`lasm_cluster_proxy_worker_count`, `lasm_cluster_proxy_queue_capacity`)
  - accept/relay batch helpers (`lasm_cluster_accept_worker_count`, `lasm_cluster_relay_accept_batch_max`, `lasm_cluster_relay_pump_batch_max`)
  - backend/connect timing helpers (`lasm_cluster_maintenance_interval_ms`, `lasm_cluster_backend_connect_timeout`, `lasm_cluster_backend_connect_cooldown`)
  - autoscale/cooldown helpers (`refresh_lasm_cluster_worker_ports_snapshot_if_changed`, `lasm_cluster_remaining_cooldown_ms`, `desired_lasm_cluster_instances`)
  - env-backed resolver helpers (`resolve_lasm_cluster_relay_accept_batch_max`, `resolve_lasm_cluster_relay_pump_batch_max`, `resolve_lasm_cluster_selection_reservation_min_chunk`)
- `main.rs` now imports these helpers through module boundaries.

## Why

Runtime config/sizing/timing logic was spread through `main.rs` and mixed with orchestration. Extracting it into one module keeps behavior unchanged while isolating configuration and resolver logic for future backend tuning.

## Result

- Smaller `main.rs` and clearer runtime ownership boundaries.
- No change to LASM cluster runtime behavior, env parsing, or diagnostics.

## Validation

- `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
- `cargo test -p sec4 --test commands run_command_lasm_cluster_status_json_skips_unchanged_snapshots`
