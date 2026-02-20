# 1179 M39 Slice: LASM Cluster Selection Lookup Sentinel Storage

This slice simplifies relay selection lookup storage representation.

## What changed

1. Selection lookup storage changed from `Vec<Option<usize>>` to `Vec<usize>`.
2. Added sentinel constant `LASM_CLUSTER_SELECTION_LOOKUP_NONE = usize::MAX` for “no backend”.
3. Lookup rebuild writes plain backend indices or sentinel values; selection read path filters sentinel back to `None`.

## Why

Using a plain index vector avoids per-entry option wrapping overhead and keeps lookup storage more compact/straightforward while preserving selection semantics.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
