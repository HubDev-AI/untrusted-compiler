# 1199 M39 Slice: LASM Cluster Non-Identity Selection Direct Lookup Read

This slice tightens relay backend-selection lookup reads on non-identity mapping paths.

## What changed

1. Non-identity selection path now reads lookup entries directly:
   - `let mapped_index = selection_lookup[start_index]`
   - sentinel check against `LASM_CLUSTER_SELECTION_LOOKUP_NONE`
2. Added debug assertion to preserve the expected lookup-size invariant in debug builds.

## Why

The previous path used `get().copied().filter(...)`. Direct lookup with sentinel check keeps behavior unchanged while removing option-chain overhead on the non-identity backend-selection hot path.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
