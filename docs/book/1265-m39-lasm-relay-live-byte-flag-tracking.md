# 1265 M39 Slice: LASM Relay Live Byte-Flag Tracking

This slice optimizes relay-shard liveness tracking representation in LASM accept/fallback dispatch paths.

## What changed

1. Added liveness byte constants:
   - `LASM_CLUSTER_RELAY_SENDER_LIVE = 1`
   - `LASM_CLUSTER_RELAY_SENDER_DEAD = 0`
2. Switched multi-sender liveness storage from `Vec<bool>` to `Vec<u8>`.
3. Updated relay liveness helpers and fallback dispatch logic to use byte-flag comparisons and writes.

## Why

`Vec<bool>` uses specialized bitset/proxy behavior that can add overhead in tight hot loops. Byte flags keep semantics explicit and can reduce liveness read/write overhead in the accept and fallback dispatch path without changing runtime behavior.

## Validation

1. `rustfmt compiler/sec4-cli/src/main.rs`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
