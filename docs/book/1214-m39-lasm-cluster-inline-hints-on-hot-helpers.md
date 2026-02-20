# 1214 M39 Slice: LASM Cluster Inline Hints on Hot Helpers

This slice adds explicit inline hints to small helper functions used in tight LASM cluster loops.

## What changed

1. Added `#[inline(always)]` to cluster counter flush helpers:
   - `flush_lasm_cluster_saturation_counters`
   - `flush_lasm_cluster_dispatch_fallback_total`
   - `flush_lasm_cluster_active_connection_increments`
   - `flush_lasm_cluster_active_connection_decrements`
2. Added `#[inline(always)]` to:
   - `dispatch_lasm_cluster_relay_stream_fallback`
   - `handle_lasm_cluster_accept_dispatch_error`

## Why

These helpers sit in frequently executed accept/relay paths. Explicit inline hints preserve decomposition for readability while reducing function-call overhead risk in release hot paths.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
