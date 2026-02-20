# 1213 M39 Slice: LASM Cluster Accept Fallback-Success Direct Enqueue

This slice removes helper dispatch handling from fallback-success outcomes in the multi-relay accept path.

## What changed

1. Fallback dispatch (`dispatch_lasm_cluster_relay_stream_fallback`) success now increments `listener_enqueued_local` directly.
2. Accept-dispatch helper handling was narrowed to error-only outcomes (`Saturated` / `Unavailable`).
3. Single-relay error handling continues through the same helper with unchanged response/flush semantics.

## Why

Fallback success is still a non-terminal success path. Handling it directly avoids extra helper invocation and result matching overhead while preserving existing saturation/unavailable behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
