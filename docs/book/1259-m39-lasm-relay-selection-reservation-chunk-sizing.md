# 1259 M39 Slice: LASM Relay Selection Reservation Chunk Sizing

This slice reduces contention on the shared relay backend-selection counter in LASM cluster relay workers.

## What changed

1. Added `LASM_CLUSTER_SELECTION_RESERVATION_MIN_CHUNK` constant (64).
2. Relay worker backend-selection reservation now uses:
   - `reservation_chunk = max(relay_accept_batch_max, LASM_CLUSTER_SELECTION_RESERVATION_MIN_CHUNK)`
   - `fetch_add(reservation_chunk)` on the shared `relay_selection_counter`.
3. Reservation window bookkeeping (`relay_selection_reservation_len`) now follows the computed chunk size.

## Why

Relay workers reserve round-robin start indices from a shared atomic counter. Reserving only `relay_accept_batch_max` can cause higher `fetch_add` frequency and avoidable cross-thread contention under load. A deterministic minimum reservation chunk lowers shared-atomic pressure while preserving existing round-robin behavior.

## Validation

1. `rustfmt compiler/sec4-cli/src/main.rs`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
