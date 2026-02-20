# 1190 M39 Slice: LASM Cluster Streaming Accept Dispatch (No Batch Buffer)

This slice removes temporary accept-batch stream buffering from the LASM cluster accept hot path.

## What changed

1. `run_lasm_cluster_accept_loop` now dispatches accepted streams immediately inside the accept loop.
2. The temporary `Vec<TcpStream>` batch (`clear` + `push` + `drain`) is removed from each iteration.
3. Dispatch semantics are preserved:
   - per-accept-worker dispatch cursor progression,
   - fallback dispatch tracking and flush,
   - saturation/unavailable response mapping and error path handling.

## Why

The previous path accepted into a temporary buffer and then drained it, which introduced avoidable per-iteration buffer churn on the hottest cluster ingress loop. Streaming accept-to-dispatch keeps the same behavior while reducing temporary container work.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
