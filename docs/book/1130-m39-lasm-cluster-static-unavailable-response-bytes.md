# 1130 M39 Slice: LASM Cluster Static Unavailable Response Bytes

This slice removes avoidable allocation/formatting overhead from LASM cluster relay failure responses.

## What changed

1. Added explicit unavailable-reason enum for relay failure envelopes:
   - `NoHealthyWorkers`
   - `WorkerUnavailable`
   - `RelaySaturated`
   - `RelayUnavailable`
2. Replaced dynamic JSON/body formatting in `write_lasm_cluster_unavailable_response(...)` with static prebuilt HTTP response bytes for each deterministic reason.
3. Updated all cluster relay failure callsites to use reason enum variants.

## Why

Under relay saturation/outage conditions, formatting JSON + HTTP envelope strings for every failure response adds avoidable hot-path work.

Using static response bytes keeps behavior deterministic while reducing allocation and formatting overhead on failure paths.

## Validation

1. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_serves_request`
2. `cargo test -p sec4 --test commands run_command_lasm_cluster_mode_forwards_db_adapter_to_workers`
