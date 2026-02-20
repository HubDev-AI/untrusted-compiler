# 1200 M39 Slice: LASM Cluster Lazy Backend Port Read on Connect Failure

This slice trims one unconditional read from the relay connect hot path.

## What changed

1. Relay worker no longer reads `selected_backend_port` before every connect attempt.
2. Backend port lookup is now performed only when connect failure warning is emitted.

## Why

On the healthy path, connect succeeds and warning logging is skipped. Delaying backend-port read to the warning path removes unnecessary success-path work while preserving existing warning diagnostics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
